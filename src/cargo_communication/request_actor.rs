use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use bsp_server::Message;
use bsp_server::RequestId;
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use crossbeam_channel::{never, select, Receiver};
use log::{info, warn};
use mockall::*;

use crate::bsp_types::notifications::{CompileTaskData, MessageType, TaskDataWithKind};
use crate::bsp_types::requests::Request;
use crate::bsp_types::StatusCode;
use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::request_actor_state::{RequestActorState, TaskState};
use crate::cargo_communication::utils::get_current_time;

pub struct RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand,
    R::Result: CargoResult,
    C: CargoHandleTrait<CargoMessage>,
{
    pub(super) sender: Box<dyn Fn(Message) + Send>,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<C>,
    pub(crate) req_id: RequestId,
    pub(super) params: R::Params,
    pub(super) root_path: PathBuf,
    pub(super) state: RequestActorState,
}

fn get_request_status_code(result: &io::Result<ExitStatus>) -> StatusCode {
    match result {
        Ok(exit_status) if exit_status.success() => StatusCode::Ok,
        _ => StatusCode::Error,
    }
}

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand,
    R::Result: CargoResult,
    C: CargoHandleTrait<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> RequestActor<R, C> {
        RequestActor {
            sender,
            cargo_handle: None,
            req_id,
            state: RequestActorState::new::<R>(params.origin_id()),
            params,
            root_path: root_path.to_path_buf(),
        }
    }

    fn next_event(&self, inbox: &Receiver<Event>) -> Option<Event> {
        let cargo_chan = self.cargo_handle.as_ref().map(|cargo| cargo.receiver());
        select! {
            recv(inbox) -> msg => msg.ok(),
            recv(cargo_chan.unwrap_or(&never())) -> msg => match msg {
                Ok(msg) => Some(Event::CargoEvent(msg)),
                Err(_) => Some(Event::CargoFinish),
            }
        }
    }

    pub fn spawn_cargo_handle(&mut self) -> Result<CargoHandle, String> {
        let command = self.params.create_command(self.root_path.clone());
        info!("Created command: {:?}", command);
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => Ok(cargo_handle),
            Err(_err) => {
                todo!()
            }
        }
    }

    pub fn run(mut self, cancel_receiver: Receiver<Event>, cargo_handle: C) {
        self.cargo_handle = Some(cargo_handle);

        self.report_task_start(self.state.root_task_id.clone(), None, None);
        self.start_compile_task();

        while let Some(event) = self.next_event(&cancel_receiver) {
            match event {
                Event::Cancel => {
                    self.cancel();
                    return;
                }
                Event::CargoFinish => {
                    self.finish_request();
                    return;
                }
                Event::CargoEvent(message) => {
                    // handle information and create notification based on that
                    match message {
                        CargoMessage::CargoStdout(msg) => self.handle_cargo_information(msg),
                        CargoMessage::CargoStderr(msg) => {
                            self.log_message(MessageType::Error, msg, None)
                        }
                    }
                }
            }
        }
    }

    fn start_compile_task(&mut self) {
        self.state.compile_state.start_time = get_current_time().unwrap();
        self.report_task_start(
            self.state.compile_state.task_id.clone(),
            None,
            // TODO change to actual BuildTargetIdentifier
            Some(TaskDataWithKind::CompileTask(CompileTaskData::default())),
        );
    }

    fn finish_request(&mut self) {
        let res = self.cargo_handle.take().unwrap().join();
        let status_code = get_request_status_code(&res);

        self.finish_execution_task(&status_code);
        self.report_task_finish(
            self.state.root_task_id.clone(),
            status_code.clone(),
            None,
            None,
        );
        self.send_response(res, &status_code);
    }

    fn finish_execution_task(&self, status_code: &StatusCode) {
        match &self.state.task_state {
            TaskState::Compile => (),
            TaskState::Run(run_state) => self.report_task_finish(
                run_state.task_id.clone(),
                status_code.clone(),
                Some("Finished target execution".to_string()),
                None,
            ),
            TaskState::Test(test_state) => self.report_task_finish(
                test_state.task_id.clone(),
                status_code.clone(),
                Some("Finished target testing".to_string()),
                None,
            ),
        }
    }

    fn cancel(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            cargo_handle.cancel();
            self.report_task_finish(
                self.state.root_task_id.clone(),
                StatusCode::Cancelled,
                None,
                None,
            );
        } else {
            warn!(
                "Tried to cancel request {} that was already finished",
                self.req_id.clone()
            );
        }
    }
}

#[automock]
pub trait CargoHandleTrait<T> {
    fn receiver(&self) -> &Receiver<T>;

    fn cancel(self);

    fn join(self) -> io::Result<ExitStatus>;
}

#[cfg(test)]
pub mod compile_request_tests {
    use std::time::Duration;

    use crate::bsp_types::requests::{Compile, CompileParams};
    use crate::cargo_communication::cargo_types::event::CargoMessage::CargoStdout;
    use bsp_server::Message;
    use cargo_metadata::diagnostic::{DiagnosticBuilder, DiagnosticSpanBuilder};
    use cargo_metadata::Message::{
        BuildFinished as BuildFinishedEnum, BuildScriptExecuted, CompilerArtifact,
        CompilerMessage as CompilerMessageEnum,
    };
    use cargo_metadata::{
        Artifact, ArtifactBuilder, ArtifactProfile, ArtifactProfileBuilder, BuildFinished,
        BuildFinishedBuilder, BuildScript, BuildScriptBuilder, CompilerMessage,
        CompilerMessageBuilder, PackageId, Target, TargetBuilder,
    };
    use crossbeam_channel::{unbounded, Sender};
    use insta::{assert_json_snapshot, Settings};

    use super::*;

    const TEST_ORIGIN_ID: &str = "test_origin_id";
    const TEST_TARGET: &str = "test_target";
    const TEST_REQ_ID: &str = "test_req_id";
    const TEST_KIND: &str = "test_kind";
    const TEST_MESSAGE: &str = "test_message";
    const RANDOM_TASK_ID: &str = "random_task_id";
    const TIMESTAMP: &str = "timestamp";
    const TEST_CRATE_TYPE: &str = "test_crate_type";
    const TEST_SRC_PATH: &str = "test_src_path";
    const TEST_OPT_LEVEL: &str = "test_opt_level";
    const TEST_PKG_ID: &str = "test_pkg_id";
    const TEST_MANIFEST_PATH: &str = "test_manifest_path";
    const TEST_FEATURE: &str = "test_feature";
    const TEST_EXECUTABLE: &str = "test_executable";
    const TEST_FILENAME: &str = "test_filename";
    const TEST_LINKED_LIB: &str = "test_linked_lib";
    const TEST_LINKED_PATH: &str = "test_linked_path";
    const TEST_CFG: &str = "test_cfg";
    const TEST_ENV: &str = "test_env";
    const TEST_OUT_DIR: &str = "test_out_dir";
    const TEST_ARGUMENTS: &str = "test_arguments";
    const TEST_ROOT_PATH: &str = "/test_root_path";

    struct ActorChannels {
        sender_to_actor: Sender<CargoMessage>,
        receiver_from_actor: Receiver<Message>,
        cancel_sender: Sender<Event>,
    }

    fn init_test(mut mock_cargo_handle: MockCargoHandleTrait<CargoMessage>) -> ActorChannels {
        let (sender_to_main, receiver_from_actor) = unbounded::<Message>();
        let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();

        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);

        let req_actor: RequestActor<Compile, MockCargoHandleTrait<CargoMessage>> =
            RequestActor::new(
                Box::new(move |msg| sender_to_main.send(msg).unwrap()),
                TEST_REQ_ID.to_string().into(),
                CompileParams {
                    targets: vec![TEST_TARGET.into()],
                    origin_id: Some(TEST_ORIGIN_ID.into()),
                    arguments: vec![TEST_ARGUMENTS.into()],
                },
                Path::new(TEST_ROOT_PATH),
            );
        let _ = jod_thread::Builder::new()
            .spawn(move || req_actor.run(cancel_receiver, mock_cargo_handle))
            .expect("failed to spawn thread")
            .detach();

        ActorChannels {
            receiver_from_actor,
            sender_to_actor,
            cancel_sender,
        }
    }

    fn no_more_msg(receiver: Receiver<Message>) {
        assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());
    }

    #[test]
    fn simple_compile() {
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        // There is no robust way to return ExitStatus hence we return Error. In consequence the
        // status code of response is 2(Error).
        mock_cargo_handle
            .expect_join()
            .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
        let channels = init_test(mock_cargo_handle);

        drop(channels.sender_to_actor);

        let mut settings = Settings::clone_current();
        settings.add_redaction(".params.eventTime", TIMESTAMP);

        settings.bind(|| {
            assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(), @r###"
            {
              "method": "build/taskStart",
              "params": {
                "eventTime": "timestamp",
                "taskId": {
                  "id": "test_origin_id"
                }
              }
            }
            "###);
        });

        settings.add_redaction(".params.taskId.id", RANDOM_TASK_ID);
        settings.bind(|| {
            assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(), @r###"
            {
              "method": "build/taskStart",
              "params": {
                "data": {
                  "target": {
                    "uri": ""
                  }
                },
                "dataKind": "compile-task",
                "eventTime": "timestamp",
                "taskId": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                }
              }
            }
            "###);
            assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(), @r###"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "status": 2,
                "taskId": {
                  "id": "random_task_id"
                }
              }
            }
            "###);
            assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(), @r###"
            {
              "id": "test_req_id",
              "result": {
                "originId": "test_origin_id",
                "statusCode": 2
              }
            }
            "###);
        });
        no_more_msg(channels.receiver_from_actor);
    }

    #[test]
    fn simple_cancel() {
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle.expect_cancel().return_const(());

        let channels = init_test(mock_cargo_handle);

        let _ = channels.receiver_from_actor.recv(); // main task started
        let _ = channels.receiver_from_actor.recv(); // compilation task started

        channels.cancel_sender.send(Event::Cancel).unwrap();

        assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(),
        {
            ".params.eventTime" => TIMESTAMP,
        }, @r###"
        {
          "method": "build/taskFinish",
          "params": {
            "eventTime": "timestamp",
            "status": 3,
            "taskId": {
              "id": "test_origin_id"
            }
          }
        }
        "###);

        no_more_msg(channels.receiver_from_actor);
    }

    #[test]
    fn compiler_artifact() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let channels = init_test(mock_cargo_handle);

        let _ = channels.receiver_from_actor.recv(); // main task started
        let _ = channels.receiver_from_actor.recv(); // compilation task started

        let compiler_artifact = default_compiler_artifact();

        channels
            .sender_to_actor
            .send(CargoStdout(CompilerArtifact(compiler_artifact)))
            .unwrap();

        assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(),
        {
            ".params.eventTime" => TIMESTAMP,
            ".params.taskId.id" => RANDOM_TASK_ID,
        }, @r###"
        {
          "method": "build/taskProgress",
          "params": {
            "eventTime": "timestamp",
            "message": "{\"package_id\":\"test_pkg_id\",\"manifest_path\":\"test_manifest_path\",\"target\":{\"name\":\"test_target\",\"kind\":[\"test_kind\"],\"crate_types\":[\"test_crate_type\"],\"required-features\":[],\"src_path\":\"test_src_path\",\"edition\":\"2015\",\"doctest\":true,\"test\":true,\"doc\":true},\"profile\":{\"opt_level\":\"test_opt_level\",\"debuginfo\":0,\"debug_assertions\":false,\"overflow_checks\":false,\"test\":false},\"features\":[\"test_feature\"],\"filenames\":[\"test_filename\"],\"executable\":\"test_executable\",\"fresh\":false}",
            "taskId": {
              "id": "random_task_id",
              "parents": [
                "test_origin_id"
              ]
            }
          }
        }
        "###
        );

        no_more_msg(channels.receiver_from_actor);
    }

    #[test]
    fn build_script_out() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let channels = init_test(mock_cargo_handle);

        let _ = channels.receiver_from_actor.recv(); // main task started
        let _ = channels.receiver_from_actor.recv(); // compilation task started

        let build_script = default_build_script();

        channels
            .sender_to_actor
            .send(CargoStdout(BuildScriptExecuted(build_script)))
            .unwrap();

        assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(),         {
            ".params.eventTime" => TIMESTAMP,
            ".params.taskId.id" => RANDOM_TASK_ID,
        },@r###"
        {
          "method": "build/taskProgress",
          "params": {
            "eventTime": "timestamp",
            "message": "{\"package_id\":\"test_pkg_id\",\"linked_libs\":[\"test_linked_lib\"],\"linked_paths\":[\"test_linked_path\"],\"cfgs\":[\"test_cfg\"],\"env\":[[\"test_env\",\"test_env\"]],\"out_dir\":\"test_out_dir\"}",
            "taskId": {
              "id": "random_task_id",
              "parents": [
                "test_origin_id"
              ]
            }
          }
        }
        "###);

        no_more_msg(channels.receiver_from_actor);
    }

    #[test]
    fn compiler_message() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let channels = init_test(mock_cargo_handle);

        let _ = channels.receiver_from_actor.recv(); // main task started
        let _ = channels.receiver_from_actor.recv(); // compilation task started

        let compiler_msg = default_compiler_message(DiagnosticLevel::Error);
        channels
            .sender_to_actor
            .send(CargoStdout(CompilerMessageEnum(compiler_msg)))
            .unwrap();

        assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(),
            {
                ".params.task.id" => RANDOM_TASK_ID,
            },@r###"
        {
          "method": "build/publishDiagnostics",
          "params": {
            "buildTarget": {
              "uri": ""
            },
            "diagnostics": [
              {
                "message": "test_message",
                "range": {
                  "end": {
                    "character": 0,
                    "line": 0
                  },
                  "start": {
                    "character": 0,
                    "line": 0
                  }
                },
                "severity": 1,
                "source": "cargo"
              }
            ],
            "originId": "test_origin_id",
            "reset": false,
            "textDocument": {
              "uri": "file:///test_root_path/test_filename"
            }
          }
        }
        "###);

        no_more_msg(channels.receiver_from_actor);
    }

    #[test]
    fn build_finished_simple() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let channels = init_test(mock_cargo_handle);

        let _ = channels.receiver_from_actor.recv(); // main task started
        let _ = channels.receiver_from_actor.recv(); // compilation task started

        let build_finished = default_build_finished();
        channels
            .sender_to_actor
            .send(CargoStdout(BuildFinishedEnum(build_finished)))
            .unwrap();

        assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(),         {
            ".params.eventTime" => TIMESTAMP,
            ".params.taskId.id" => RANDOM_TASK_ID,
            ".params.data.time" => 0,
        },@r###"
        {
          "method": "build/taskFinish",
          "params": {
            "data": {
              "errors": 0,
              "originId": "test_origin_id",
              "target": {
                "uri": ""
              },
              "time": 0,
              "warnings": 0
            },
            "dataKind": "compile-report",
            "eventTime": "timestamp",
            "status": 1,
            "taskId": {
              "id": "random_task_id",
              "parents": [
                "test_origin_id"
              ]
            }
          }
        }
        "###);

        no_more_msg(channels.receiver_from_actor);
    }

    #[test]
    fn build_finished_with_complex_compile_report() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let channels = init_test(mock_cargo_handle);

        let _ = channels.receiver_from_actor.recv(); // main task started
        let _ = channels.receiver_from_actor.recv(); // compilation task started

        let compiler_msg_warning = default_compiler_message(DiagnosticLevel::Warning);
        let compiler_msg_error = default_compiler_message(DiagnosticLevel::Error);

        channels
            .sender_to_actor
            .send(CargoStdout(CompilerMessageEnum(compiler_msg_warning)))
            .unwrap();
        channels
            .sender_to_actor
            .send(CargoStdout(CompilerMessageEnum(compiler_msg_error)))
            .unwrap();

        let _ = channels.receiver_from_actor.recv(); // publish diagnostic
        let _ = channels.receiver_from_actor.recv(); // publish diagnostic

        let build_finished = default_build_finished();
        channels
            .sender_to_actor
            .send(CargoStdout(BuildFinishedEnum(build_finished)))
            .unwrap();

        assert_json_snapshot!(channels.receiver_from_actor.recv().unwrap(),         {
            ".params.eventTime" => TIMESTAMP,
            ".params.taskId.id" => RANDOM_TASK_ID,
            ".params.data.time" => 0,
        },@r###"
        {
          "method": "build/taskFinish",
          "params": {
            "data": {
              "errors": 1,
              "originId": "test_origin_id",
              "target": {
                "uri": ""
              },
              "time": 0,
              "warnings": 1
            },
            "dataKind": "compile-report",
            "eventTime": "timestamp",
            "status": 1,
            "taskId": {
              "id": "random_task_id",
              "parents": [
                "test_origin_id"
              ]
            }
          }
        }
        "###);

        no_more_msg(channels.receiver_from_actor);
    }

    fn default_target() -> Target {
        TargetBuilder::default()
            .name(TEST_TARGET.to_string())
            .kind(vec![TEST_KIND.into()])
            .crate_types(vec![TEST_CRATE_TYPE.into()])
            .src_path(TEST_SRC_PATH.to_string())
            .build()
            .unwrap()
    }

    fn default_artifact_profile() -> ArtifactProfile {
        ArtifactProfileBuilder::default()
            .opt_level(TEST_OPT_LEVEL.to_string())
            .debuginfo(Some(0))
            .debug_assertions(false)
            .overflow_checks(false)
            .test(false)
            .build()
            .unwrap()
    }

    fn default_compiler_artifact() -> Artifact {
        ArtifactBuilder::default()
            .package_id(PackageId {
                repr: TEST_PKG_ID.into(),
            })
            .manifest_path(TEST_MANIFEST_PATH.to_string())
            .target(default_target())
            .profile(default_artifact_profile())
            .executable(Some(TEST_EXECUTABLE.into()))
            .features(vec![TEST_FEATURE.into()])
            .filenames(vec![TEST_FILENAME.into()])
            .fresh(false)
            .build()
            .unwrap()
    }

    fn default_build_script() -> BuildScript {
        BuildScriptBuilder::default()
            .package_id(PackageId {
                repr: TEST_PKG_ID.into(),
            })
            .linked_libs(vec![TEST_LINKED_LIB.into()])
            .linked_paths(vec![TEST_LINKED_PATH.into()])
            .cfgs(vec![TEST_CFG.into()])
            .env(vec![(TEST_ENV.into(), TEST_ENV.into())])
            .out_dir(TEST_OUT_DIR.to_string())
            .build()
            .unwrap()
    }

    fn default_diagnostic_span() -> DiagnosticSpan {
        DiagnosticSpanBuilder::default()
            .file_name(TEST_FILENAME.to_string())
            .byte_start(0_u32)
            .byte_end(0_u32)
            .line_start(0_usize)
            .line_end(0_usize)
            .column_start(0_usize)
            .column_end(0_usize)
            .is_primary(true)
            .text(vec![])
            .label(None)
            .suggested_replacement(None)
            .suggestion_applicability(None)
            .expansion(None)
            .build()
            .unwrap()
    }

    fn default_diagnostic(level: DiagnosticLevel) -> Diagnostic {
        DiagnosticBuilder::default()
            .message(TEST_MESSAGE.to_string())
            .level(level)
            .code(None)
            .spans(vec![default_diagnostic_span()])
            .children(vec![])
            .rendered(None)
            .build()
            .unwrap()
    }

    fn default_compiler_message(level: DiagnosticLevel) -> CompilerMessage {
        CompilerMessageBuilder::default()
            .package_id(PackageId {
                repr: TEST_PKG_ID.into(),
            })
            .target(default_target())
            .message(default_diagnostic(level))
            .build()
            .unwrap()
    }

    fn default_build_finished() -> BuildFinished {
        BuildFinishedBuilder::default()
            .success(true)
            .build()
            .unwrap()
    }
}
