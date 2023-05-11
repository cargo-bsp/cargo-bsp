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
use log::warn;
use mockall::*;

use bsp_types::notifications::{CompileTaskData, MessageType, TaskDataWithKind};
use bsp_types::requests::Request;
use bsp_types::StatusCode;

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
    C: CargoHandler<CargoMessage>,
{
    pub(super) sender: Box<dyn Fn(Message) + Send>,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<C>,
    cancel_receiver: Receiver<Event>,
    pub(super) req_id: RequestId,
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
    C: CargoHandler<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
        cargo_handle: C,
        cancel_receiver: Receiver<Event>,
    ) -> RequestActor<R, C> {
        RequestActor {
            sender,
            cargo_handle: Some(cargo_handle),
            cancel_receiver,
            req_id,
            state: RequestActorState::new::<R>(params.origin_id()),
            params,
            root_path: root_path.to_path_buf(),
        }
    }

    fn next_event(&self) -> Option<Event> {
        let cargo_chan = self.cargo_handle.as_ref().map(|cargo| cargo.receiver());
        select! {
            recv(self.cancel_receiver) -> msg => msg.ok(),
            recv(cargo_chan.unwrap_or(&never())) -> msg => match msg {
                Ok(msg) => Some(Event::CargoEvent(msg)),
                Err(_) => Some(Event::CargoFinish),
            }
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::CargoEvent(message) => {
                // handle information and create notification based on that
                match message {
                    CargoMessage::CargoStdout(msg) => self.handle_cargo_information(msg),
                    CargoMessage::CargoStderr(msg) => {
                        self.log_message(MessageType::Error, msg, None)
                    }
                }
            }
            Event::Cancel => {
                self.cancel();
            }
            Event::CargoFinish => {
                self.finish_request();
            }
        }
    }

    pub fn run(mut self) {
        self.report_task_start(self.state.root_task_id.clone(), None, None);
        self.start_compile_task();

        while let Some(event) = self.next_event() {
            self.handle_event(event);
        }
    }

    fn start_compile_task(&mut self) {
        self.state.compile_state.start_time = get_current_time();
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
pub trait CargoHandler<T> {
    fn receiver(&self) -> &Receiver<T>;

    fn cancel(self);

    fn join(self) -> io::Result<ExitStatus>;
}

#[cfg(test)]
pub mod compile_request_tests {
    use bsp_server::Message;
    use crossbeam_channel::unbounded;
    use insta::{assert_json_snapshot, Settings};

    use bsp_types::requests::{Compile, CompileParams};
    use bsp_types::BuildTargetIdentifier;

    use crate::utils::tests::no_more_msg;

    use super::*;

    const TEST_ORIGIN_ID: &str = "test_origin_id";
    const TEST_TARGET: &str = "test_target";
    const TEST_REQ_ID: &str = "test_req_id";
    const RANDOM_TASK_ID: &str = "random_task_id";
    const TIMESTAMP: &str = "timestamp";
    const TEST_ARGUMENTS: &str = "test_arguments";
    const TEST_ROOT_PATH: &str = "/test_root_path";

    fn default_req_actor(
        cargo_handle: MockCargoHandler<CargoMessage>,
    ) -> (
        RequestActor<Compile, MockCargoHandler<CargoMessage>>,
        Receiver<Message>,
    ) {
        let (sender_to_main, receiver_from_actor) = unbounded::<Message>();
        let (_cancel_sender, cancel_receiver) = unbounded::<Event>();

        (
            RequestActor::new(
                Box::new(move |msg| sender_to_main.send(msg).unwrap()),
                TEST_REQ_ID.to_string().into(),
                CompileParams {
                    targets: vec![BuildTargetIdentifier {
                        uri: TEST_TARGET.into(),
                    }],
                    origin_id: Some(TEST_ORIGIN_ID.into()),
                    arguments: vec![TEST_ARGUMENTS.into()],
                },
                Path::new(TEST_ROOT_PATH),
                cargo_handle,
                cancel_receiver,
            ),
            receiver_from_actor,
        )
    }

    #[test]
    fn compile_lifetime() {
        let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();

        let mut mock_cargo_handle = MockCargoHandler::new();
        // There is no robust way to return ExitStatus hence we return Error. In consequence the
        // status code of response is 2(Error).
        mock_cargo_handle
            .expect_join()
            .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);
        let (req_actor, receiver_from_actor) = default_req_actor(mock_cargo_handle);

        // The channel is closed so the actor finishes its execution.
        drop(sender_to_actor);

        req_actor.run();

        let mut settings = Settings::clone_current();
        settings.add_redaction(".params.eventTime", TIMESTAMP);
        settings.bind(|| {
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r###"
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
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r###"
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
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r###"
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
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r###"
            {
              "id": "test_req_id",
              "result": {
                "originId": "test_origin_id",
                "statusCode": 2
              }
            }
            "###);
        });
        no_more_msg(receiver_from_actor);
    }

    #[test]
    fn cancel_with_cargo_handle() {
        let mut mock_cargo_handle = MockCargoHandler::new();
        mock_cargo_handle.expect_cancel().return_const(());

        let (mut req_actor, receiver_from_actor) = default_req_actor(mock_cargo_handle);

        req_actor.cancel();

        assert_json_snapshot!(receiver_from_actor.recv().unwrap(),
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

        no_more_msg(receiver_from_actor);
    }

    #[test]
    fn multiple_cancel() {
        // Client should receive only one cancel message.

        let mut mock_cargo_handle = MockCargoHandler::new();
        mock_cargo_handle.expect_cancel().return_const(());

        let (mut req_actor, receiver_from_actor) = default_req_actor(mock_cargo_handle);

        req_actor.cancel();
        req_actor.cancel();
        req_actor.cancel();

        let _ = receiver_from_actor.recv().unwrap();
        no_more_msg(receiver_from_actor);
    }

    mod cargo_messages_tests {
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

        use crate::cargo_communication::cargo_types::event::CargoMessage::CargoStdout;
        use crate::cargo_communication::cargo_types::event::Event::CargoEvent;

        use super::*;

        const TEST_KIND: &str = "test_kind";
        const TEST_MESSAGE: &str = "test_message";
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

        #[test]
        fn compiler_artifact() {
            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStdout(CompilerArtifact(
                default_compiler_artifact(),
            ))));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),
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

            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn build_script_out() {
            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStdout(BuildScriptExecuted(
                default_build_script(),
            ))));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),         {
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

            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn compiler_message() {
            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStdout(CompilerMessageEnum(
                default_compiler_message(DiagnosticLevel::Error),
            ))));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),
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

            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn build_finished_simple() {
            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStdout(BuildFinishedEnum(
                default_build_finished(),
            ))));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),         {
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

            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn build_finished_with_complex_compile_report() {
            // Checks if server counts warnings and error and produces a correct compile report.

            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStdout(CompilerMessageEnum(
                default_compiler_message(DiagnosticLevel::Error),
            ))));
            req_actor.handle_event(CargoEvent(CargoStdout(CompilerMessageEnum(
                default_compiler_message(DiagnosticLevel::Warning),
            ))));

            let _ = receiver_from_actor.recv(); // publish diagnostic
            let _ = receiver_from_actor.recv(); // publish diagnostic

            req_actor.handle_event(CargoEvent(CargoStdout(BuildFinishedEnum(
                default_build_finished(),
            ))));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),         {
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

            no_more_msg(receiver_from_actor);
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

    mod run_request_tests {
        use super::*;
        use crate::bsp_types::requests::{Run, RunParams};
        use crate::bsp_types::BuildTargetIdentifier;
        use crate::cargo_communication::cargo_types::event::CargoMessage::{
            CargoStderr, CargoStdout,
        };
        use crate::cargo_communication::cargo_types::event::Event::CargoEvent;
        use bsp_server::Message;
        use cargo_metadata::Message::{BuildFinished as BuildFinishedEnum, TextLine};
        use cargo_metadata::{BuildFinished, BuildFinishedBuilder};
        use crossbeam_channel::unbounded;

        fn default_req_actor(
            cargo_handle: MockCargoHandler<CargoMessage>,
        ) -> (
            RequestActor<Run, MockCargoHandler<CargoMessage>>,
            Receiver<Message>,
        ) {
            let (sender_to_main, receiver_from_actor) = unbounded::<Message>();
            let (_cancel_sender, cancel_receiver) = unbounded::<Event>();

            (
                RequestActor::new(
                    Box::new(move |msg| sender_to_main.send(msg).unwrap()),
                    TEST_REQ_ID.to_string().into(),
                    RunParams {
                        target: BuildTargetIdentifier {
                            uri: TEST_TARGET.into(),
                        },
                        origin_id: Some(TEST_ORIGIN_ID.into()),
                        arguments: vec![TEST_ARGUMENTS.into()],
                        data_kind: None,
                        data: None,
                    },
                    Path::new(TEST_ROOT_PATH),
                    cargo_handle,
                    cancel_receiver,
                ),
                receiver_from_actor,
            )
        }

        fn default_build_finished() -> BuildFinished {
            BuildFinishedBuilder::default()
                .success(true)
                .build()
                .unwrap()
        }

        #[ignore]
        #[test]
        fn simple_run() {
            #[allow(unused_mut)]
            let mut mock_cargo_handle = MockCargoHandler::new();
            mock_cargo_handle
                .expect_join()
                .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
            let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();
            mock_cargo_handle
                .expect_receiver()
                .return_const(receiver_from_cargo);

            let (req_actor, receiver_from_actor) = default_req_actor(mock_cargo_handle);

            let _ = jod_thread::Builder::new()
                .spawn(move || req_actor.run())
                .expect("failed to spawn thread")
                .detach();

            println!(
                "{:?}",
                receiver_from_actor
                    .recv()
                    .unwrap_or_else(|e| panic!("Failed to receive message: {}", e))
            );
            println!(
                "{:?}",
                receiver_from_actor
                    .recv()
                    .unwrap_or_else(|e| panic!("Failed to receive message: {}", e))
            );

            // let _ = receiver_from_actor.recv(); // main task started
            // let _ = receiver_from_actor.recv(); // compilation task started

            sender_to_actor
                .send(CargoStdout(BuildFinishedEnum(default_build_finished())))
                .unwrap_or_else(|e| panic!("Failed to send message: {}", e));

            // let _ = receiver_from_actor.recv(); // compilation task finished
            println!(
                "{:?}",
                receiver_from_actor
                    .recv()
                    .unwrap_or_else(|e| panic!("Failed to receive message: {}", e))
            );

            assert_json_snapshot!(receiver_from_actor.recv().unwrap_or_else(|e|
            panic!("Failed to receive message: {}", e)
            ),{
            ".params.taskId.id" => RANDOM_TASK_ID,
            ".params.taskId.parents" => format!("[{TEST_ORIGIN_ID}]"),
            ".params.eventTime" => TIMESTAMP,
            }
            ,@r###"
            {
              "method": "build/taskStart",
              "params": {
                "eventTime": "timestamp",
                "message": "Started target execution",
                "taskId": {
                  "id": "random_task_id",
                  "parents": "[test_origin_id]"
                }
              }
            }
            "###);

            assert_json_snapshot!(receiver_from_actor.recv().unwrap_or_else(|e|
            panic!("Failed to receive message: {}", e)
            ),{
            ".params.taskId.id" => RANDOM_TASK_ID,
            ".params.taskId.parents" => format!("[{TEST_ORIGIN_ID}]"),
            ".params.eventTime" => TIMESTAMP,
            }
            ,@r###"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "message": "Finished target execution",
                "status": 2,
                "taskId": {
                  "id": "random_task_id",
                  "parents": "[test_origin_id]"
                }
              }
            }
            "###);
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
            ".params.taskId.id" => TEST_ORIGIN_ID,
            ".params.eventTime" => TIMESTAMP,
            }
            ,@r###"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "status": 2,
                "taskId": {
                  "id": "test_origin_id"
                }
              }
            }
            "###);
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
            ".result.originId" => TEST_ORIGIN_ID,
            }
            ,@r###"
            {
              "id": "test_req_id",
              "result": {
                "originId": "test_origin_id",
                "statusCode": 2
              }
            }
            "###);

            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn simple_stdout() {
            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStdout(TextLine(
                "test_text_line".to_string(),
            ))));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
            ".params.task.id" => RANDOM_TASK_ID,
            ".params.taskId.parents" => format!("[{TEST_ORIGIN_ID}]"),
            } ,@r###"
            {
              "method": "build/logMessage",
              "params": {
                "message": "test_text_line",
                "originId": "test_origin_id",
                "task": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                },
                "type": 4
              }
            }
            "###);

            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn simple_stderr() {
            let (mut req_actor, receiver_from_actor) = default_req_actor(MockCargoHandler::new());

            req_actor.handle_event(CargoEvent(CargoStderr("test_stderr".to_string())));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
            ".params.task.id" => RANDOM_TASK_ID,
            ".params.taskId.parents" => format!("[{TEST_ORIGIN_ID}]"),
            } ,@r###"
            {
              "method": "build/logMessage",
              "params": {
                "message": "test_stderr",
                "originId": "test_origin_id",
                "task": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                },
                "type": 1
              }
            }
            "###);

            no_more_msg(receiver_from_actor);
        }
    }
}
