//! Handles messages from Cargo build/run/test commands, parsing them and preparing appropriate
//! notifications and responses. Also handles information about the finish of
//! Cargo command and the cancel request from the client.
//!
//! The hierarchy of tasks:
//! - for every request there is a root task with no `data-kind` and its subtask
//! (or subtasks if there are more than one build targets) informing about the compilation
//! process with `data-kind` `compile-task` and `compile-report` (for `taskStart` and `taskFinish`)
//! - for every task, before running the requested command, the unit graph command is
//! executed, which starts a subtask (to root task) with no `data-kind`. This subtask finishes
//! before the execution of the requested command, and therefore, before starting any other subtasks
//! - for `run` and `test` requests there is an additional subtask (to root task) for
//! execution process (with no `data-kind`) (started after the compilation subtask/s)
//! - additionally for `test` requests there can be subtasks (to execution task) for each
//! test suite with `test-task` and `test-report` `data-kind`. These subtasks may have additional
//! subtasks for each single test with `test-start` and `test-finish` `data-kind`.
//! Additionally, for each requests there may be `logMessage` notifications for stdout/stderr
//! messages from Cargo and `publishDiagnostic` notifications.
//!
//! We assume that test suites are executed one after another in specific order,
//! see [`cargo_types/test.rs`].

use std::path::{Path, PathBuf};

use bsp_server::Message;
use bsp_server::RequestId;
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use cargo_metadata::Message as CargoMetadataMessage;
use crossbeam_channel::{never, select, Receiver};
use log::warn;

use serde::Deserialize;

use crate::cargo_communication::cargo_handle::CargoHandler;
use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::cargo_types::params_target::ParamsTarget;
use crate::cargo_communication::execution::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::execution::cargo_types::cargo_unit_graph_command::CreateUnitGraphCommand;
use crate::cargo_communication::execution::cargo_types::origin_id::OriginId;
use crate::cargo_communication::execution::execution_actor_state::{
    ExecutionActorState, TaskState,
};
use crate::project_model::workspace::{ProjectWorkspace, SrcPathToTargetId};
use bsp_types::notifications::{CompileTaskData, MessageType, TaskDataWithKind};
use bsp_types::requests::Request;
use bsp_types::{BuildTargetIdentifier, StatusCode};

pub(crate) struct ExecutionActor<R, C>
where
    R: Request,
    R::Params: CreateUnitGraphCommand,
    R::Result: CargoResult,
    C: CargoHandler<CargoMessage>,
{
    // sender for notifications and responses to main loop
    pub(super) sender: Box<dyn Fn(Message) + Send>,
    pub(super) cargo_handle: Option<C>,
    cancel_receiver: Receiver<Event>,
    pub(super) req_id: RequestId,
    pub(super) params: R::Params,
    pub(super) root_path: PathBuf,
    pub(super) build_targets: Vec<BuildTargetIdentifier>,
    pub(super) src_path_to_target_id: SrcPathToTargetId,
    pub(super) state: ExecutionActorState,
}

impl<R, C> ExecutionActor<R, C>
where
    R: Request,
    R::Params: CreateUnitGraphCommand + ParamsTarget + OriginId,
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
        workspace: &ProjectWorkspace,
    ) -> ExecutionActor<R, C> {
        let build_targets = params.get_targets(workspace);
        ExecutionActor {
            sender,
            cargo_handle: Some(cargo_handle),
            cancel_receiver,
            req_id,
            state: ExecutionActorState::new::<R>(params.origin_id(), &build_targets),
            build_targets,
            params,
            root_path: root_path.to_path_buf(),
            src_path_to_target_id: workspace.src_path_to_target_id.clone(),
        }
    }

    pub fn next_event(&self) -> Option<Event> {
        let cargo_chan = self.cargo_handle.as_ref().map(|cargo| cargo.receiver());
        select! {
            recv(self.cancel_receiver) -> msg => msg.ok(),
            recv(cargo_chan.unwrap_or(&never())) -> msg => match msg {
                Ok(msg) => Some(Event::CargoEvent(msg)),
                Err(_) => Some(Event::CargoFinish),
            }
        }
    }

    fn handle_cargo_event(&mut self, message: CargoMessage) {
        // handle information and create notification based on that
        match message {
            CargoMessage::CargoStdout(msg) => self.deserialize_and_handle_cargo_information(msg),
            CargoMessage::CargoStderr(msg) => self.log_message(MessageType::Error, msg, None),
        }
    }

    /// Main function of RequestActor, invoked for every new compile/run/test request.
    pub fn run(mut self) {
        self.start_compile_task();

        while let Some(event) = self.next_event() {
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
                    self.handle_cargo_event(message);
                }
            }
        }
    }

    fn start_compile_task(&mut self) {
        self.report_task_start(
            self.state.compile_state.task_id.clone(),
            Some("Started compilation".to_string()),
            None,
        );
        self.build_targets.iter().for_each(|id| {
            self.state.compile_state.set_start_time(id);
        });
        self.build_targets.iter().for_each(|id| {
            self.report_task_start(
                self.state.compile_state.get_target_task_id(id),
                None,
                Some(TaskDataWithKind::CompileTask(CompileTaskData {
                    target: id.clone(),
                })),
            );
        });
    }

    fn deserialize_and_handle_cargo_information(&mut self, msg: String) {
        let mut deserializer = serde_json::Deserializer::from_str(&msg);
        let message = CargoMetadataMessage::deserialize(&mut deserializer)
            .unwrap_or(CargoMetadataMessage::TextLine(msg));
        self.handle_cargo_information(message);
    }

    fn finish_request(&mut self) {
        let command_result = self.cargo_handle.take().unwrap().join();

        self.finish_execution_task();
        self.report_task_finish(self.state.root_task_id.clone(), StatusCode::Ok, None, None);
        self.send_response(command_result);
    }

    fn finish_execution_task(&self) {
        match &self.state.task_state {
            TaskState::Compile => (),
            TaskState::Run(run_state) => self.report_task_finish(
                run_state.task_id.clone(),
                StatusCode::Ok,
                Some("Finished target execution".to_string()),
                None,
            ),
            TaskState::Test(test_state) => self.report_task_finish(
                test_state.task_id.clone(),
                StatusCode::Ok,
                Some("Finished target testing".to_string()),
                None,
            ),
        }
    }

    // TODO: Cancel should cancel all started tasks, not only the root task.
    pub fn cancel(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            cargo_handle.cancel();
            self.send_cancel_response();
        } else {
            warn!(
                "Tried to cancel request {} that was already finished",
                self.req_id.clone()
            );
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::cargo_communication::cargo_handle::MockCargoHandler;
    use crate::cargo_communication::utils::{test_package, test_target, test_target_id};
    use crate::utils::tests::no_more_msg;
    use bsp_server::Message;
    use bsp_types::requests::{Compile, CompileParams};
    use bsp_types::BuildTargetIdentifier;
    use cargo_metadata::camino::Utf8PathBuf;
    use cargo_metadata::Message::BuildFinished as BuildFinishedEnum;
    use cargo_metadata::{BuildFinished, BuildFinishedBuilder};
    use crossbeam_channel::{unbounded, Sender};
    use insta::{assert_json_snapshot, Settings};
    use std::collections::HashMap;

    const TEST_ORIGIN_ID: &str = "test_origin_id";
    const TEST_TARGET: &str = "test_target";
    const TEST_TARGET2: &str = "test_target2";
    const TEST_REQ_ID: &str = "test_req_id";
    const RANDOM_TASK_ID: &str = "random_task_id";
    const TIMESTAMP: &str = "timestamp";
    const TEST_ARGUMENTS: &str = "test_arguments";
    const TEST_ROOT_PATH: &str = "/test_root_path";
    const TEST_PACKAGE: &str = "test_package";
    const BIN_KIND: &str = "bin";
    const LIB_KIND: &str = "lib";
    const TEST_SRC_PATH: &str = "test_src_path";

    enum TestCase {
        OneTarget,
        MultipleTargets,
    }

    // Struct that contains all the endpoints needed for testing
    struct TestEndpoints<R>
    where
        R: Request,
        R::Params: CreateUnitGraphCommand,
        R::Result: CargoResult,
    {
        req_actor: ExecutionActor<R, MockCargoHandler<CargoMessage>>,
        receiver_from_actor: Receiver<Message>,
        _cancel_sender: Sender<Event>,
    }

    fn default_req_actor<R>(
        cargo_handle: MockCargoHandler<CargoMessage>,
        params: R::Params,
        test_case: TestCase,
    ) -> TestEndpoints<R>
    where
        R: Request,
        R::Params: CreateUnitGraphCommand + ParamsTarget + OriginId,
        R::Result: CargoResult,
    {
        let (sender_to_main, receiver_from_actor) = unbounded::<Message>();
        // Cancel sender needs to be referenced to avoid closing the channel
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();
        let workspace = match test_case {
            TestCase::OneTarget => test_simple_workspace(),
            TestCase::MultipleTargets => test_complex_workspace(),
        };
        TestEndpoints {
            req_actor: ExecutionActor::new(
                Box::new(move |msg| sender_to_main.send(msg).unwrap()),
                TEST_REQ_ID.to_string().into(),
                params,
                Path::new(TEST_ROOT_PATH),
                cargo_handle,
                cancel_receiver,
                &workspace,
            ),
            receiver_from_actor,
            _cancel_sender: cancel_sender,
        }
    }

    fn default_build_finished() -> BuildFinished {
        BuildFinishedBuilder::default()
            .success(true)
            .build()
            .unwrap()
    }

    fn test_simple_workspace() -> ProjectWorkspace {
        let packages = vec![test_package(TEST_PACKAGE)];
        let test_target_id = test_target_id(TEST_TARGET);

        let target_id_to_package_name =
            HashMap::from([(test_target_id.clone(), TEST_PACKAGE.to_string())]);
        let target_id_to_target_data =
            HashMap::from([(test_target_id.clone(), test_target(TEST_TARGET, BIN_KIND))]);
        let src_path_to_target_id =
            HashMap::from([(Utf8PathBuf::from(TEST_SRC_PATH), test_target_id)]);
        ProjectWorkspace {
            packages,
            target_id_to_package_name,
            target_id_to_target_data,
            src_path_to_target_id,
        }
    }

    fn test_complex_workspace() -> ProjectWorkspace {
        let mut workspace = test_simple_workspace();
        let test_target_id = test_target_id(TEST_TARGET2);
        workspace
            .target_id_to_package_name
            .insert(test_target_id.clone(), TEST_PACKAGE.to_string());
        workspace
            .target_id_to_target_data
            .insert(test_target_id.clone(), test_target(TEST_TARGET2, LIB_KIND));
        workspace
            .src_path_to_target_id
            .insert(Utf8PathBuf::from(TEST_SRC_PATH), test_target_id);
        workspace
    }

    mod compile_request_tests {
        use super::*;
        use crate::cargo_communication::utils::test_target_id;
        use std::io;

        fn default_compile_params(test_case: TestCase) -> CompileParams {
            let mut targets = vec![test_target_id(TEST_TARGET)];
            if let TestCase::MultipleTargets = test_case {
                targets.push(test_target_id(TEST_TARGET2));
            }
            CompileParams {
                targets,
                origin_id: Some(TEST_ORIGIN_ID.into()),
                arguments: vec![TEST_ARGUMENTS.into()],
            }
        }

        fn mock_cargo_handler(
            receiver_from_cargo: Receiver<CargoMessage>,
        ) -> TestEndpoints<Compile> {
            let mut mock_cargo_handle = MockCargoHandler::new();
            // There is no robust way to return ExitStatus hence we return Error. In consequence the
            // status code of response is 2(Error).
            mock_cargo_handle
                .expect_join()
                .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
            mock_cargo_handle
                .expect_receiver()
                .return_const(receiver_from_cargo);
            default_req_actor::<Compile>(
                mock_cargo_handle,
                default_compile_params(TestCase::MultipleTargets),
                TestCase::MultipleTargets,
            )
        }

        mod unit_graph_tests {
            use super::*;
            use crate::cargo_communication::execution::cargo_types::unit_graph::UnitGraph;
            use serde_json::to_string;

            #[test]
            fn unit_graph_error() {
                let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();

                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = mock_cargo_handler(receiver_from_cargo);

                let _ = jod_thread::Builder::new()
                    .spawn(move || req_actor.run_unit_graph())
                    .expect("failed to spawn thread")
                    .detach();

                // The channel is closed so the actor finishes its execution.
                drop(sender_to_actor);

                let mut settings = Settings::clone_current();
                settings.add_redaction(".params.eventTime", TIMESTAMP);
                settings.add_redaction(".params.taskId.id", RANDOM_TASK_ID);
                settings.bind(|| {
                    assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
                {
                  "method": "build/taskStart",
                  "params": {
                    "eventTime": "timestamp",
                    "message": "Started unit graph command",
                    "taskId": {
                      "id": "random_task_id",
                      "parents": [
                        "test_origin_id"
                      ]
                    }
                  }
                }
                "#);
                    assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "eventTime": "timestamp",
                    "message": "Finished unit graph command",
                    "status": 2,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": [
                        "test_origin_id"
                      ]
                    }
                  }
                }
                "#);
                });
                no_more_msg(receiver_from_actor);
            }

            #[test]
            fn unit_graph_success() {
                let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();

                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = mock_cargo_handler(receiver_from_cargo);

                let _ = jod_thread::Builder::new()
                    .spawn(move || req_actor.run_unit_graph())
                    .expect("failed to spawn thread")
                    .detach();

                let _ = receiver_from_actor.recv().unwrap(); // unit graph task started

                sender_to_actor
                    .send(CargoMessage::CargoStdout(
                        to_string(&UnitGraph::default()).unwrap(),
                    ))
                    .unwrap();

                drop(sender_to_actor);

                assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.eventTime" => TIMESTAMP,
                }
                ,@r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "eventTime": "timestamp",
                    "message": "Finished unit graph command",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": [
                        "test_origin_id"
                      ]
                    }
                  }
                }
                "#);
                no_more_msg(receiver_from_actor);
            }
        }

        #[test]
        fn compile_lifetime() {
            let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();

            let TestEndpoints {
                req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = mock_cargo_handler(receiver_from_cargo);

            let _ = jod_thread::Builder::new()
                .spawn(move || req_actor.run())
                .expect("failed to spawn thread")
                .detach();

            let mut settings = Settings::clone_current();
            settings.add_redaction(".params.eventTime", TIMESTAMP);

            // The channel is closed so the actor finishes its execution.
            drop(sender_to_actor);

            settings.add_redaction(".params.taskId.id", RANDOM_TASK_ID);
            settings.bind(|| {
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
                {
                  "method": "build/taskStart",
                  "params": {
                    "eventTime": "timestamp",
                    "message": "Started compilation",
                    "taskId": {
                      "id": "random_task_id",
                      "parents": [
                        "test_origin_id"
                      ]
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                }
                ,@r#"
                {
                  "method": "build/taskStart",
                  "params": {
                    "data": {
                      "target": {
                        "uri": "test_target"
                      }
                    },
                    "dataKind": "compile-task",
                    "eventTime": "timestamp",
                    "taskId": {
                      "id": "random_task_id",
                      "parents": "[random_task_id]"
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                }
                ,@r#"
                {
                  "method": "build/taskStart",
                  "params": {
                    "data": {
                      "target": {
                        "uri": "test_target2"
                      }
                    },
                    "dataKind": "compile-task",
                    "eventTime": "timestamp",
                    "taskId": {
                      "id": "random_task_id",
                      "parents": "[random_task_id]"
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "eventTime": "timestamp",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id"
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
                {
                  "id": "test_req_id",
                  "error": {
                    "code": -32603,
                    "message": "other error"
                  }
                }
                "#);
            });
            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn cancel_with_cargo_handle() {
            let mut mock_cargo_handle = MockCargoHandler::new();
            mock_cargo_handle.expect_cancel().return_const(());

            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Compile>(
                mock_cargo_handle,
                default_compile_params(TestCase::OneTarget),
                TestCase::OneTarget,
            );

            req_actor.cancel();

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),
            {
                ".params.eventTime" => TIMESTAMP,
            }, @r#"
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
            "#);
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
            {
              "id": "test_req_id",
              "error": {
                "code": -32800,
                "message": "canceled by client"
              }
            }
            "#);
            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn multiple_cancel() {
            // Client should receive only one cancel message.

            let mut mock_cargo_handle = MockCargoHandler::new();
            mock_cargo_handle.expect_cancel().return_const(());

            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Compile>(
                mock_cargo_handle,
                default_compile_params(TestCase::OneTarget),
                TestCase::OneTarget,
            );

            req_actor.cancel();
            req_actor.cancel();
            req_actor.cancel();

            let _ = receiver_from_actor.recv().unwrap(); // main task notification
            let _ = receiver_from_actor.recv().unwrap(); // response
            no_more_msg(receiver_from_actor);
        }

        mod cargo_compile_messages_tests {
            use super::*;
            use cargo_metadata::diagnostic::{DiagnosticBuilder, DiagnosticSpanBuilder};
            use cargo_metadata::Message::{
                BuildFinished as BuildFinishedEnum, BuildScriptExecuted, CompilerArtifact,
                CompilerMessage as CompilerMessageEnum,
            };
            use cargo_metadata::{
                Artifact, ArtifactBuilder, ArtifactDebuginfo, ArtifactProfile,
                ArtifactProfileBuilder, BuildScript, BuildScriptBuilder, CompilerMessage,
                CompilerMessageBuilder, PackageId, Target, TargetBuilder,
            };

            const TEST_KIND: &str = "test_kind";
            const TEST_MESSAGE: &str = "test_message";
            const TEST_CRATE_TYPE: &str = "test_crate_type";
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
                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = default_req_actor::<Compile>(
                    MockCargoHandler::new(),
                    default_compile_params(TestCase::OneTarget),
                    TestCase::OneTarget,
                );

                req_actor.handle_cargo_information(CompilerArtifact(default_compiler_artifact()));

                assert_json_snapshot!(receiver_from_actor.recv().unwrap(),
                {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                }, @r#"
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
                    },
                    "unit": "compilation_steps"
                  }
                }
                "#
                );
                no_more_msg(receiver_from_actor);
            }

            #[test]
            fn build_script_out() {
                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = default_req_actor::<Compile>(
                    MockCargoHandler::new(),
                    default_compile_params(TestCase::OneTarget),
                    TestCase::OneTarget,
                );

                req_actor.handle_cargo_information(BuildScriptExecuted(default_build_script()));

                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                },@r#"
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
                    },
                    "unit": "compilation_steps"
                  }
                }
                "#);
                no_more_msg(receiver_from_actor);
            }

            #[test]
            fn compiler_message() {
                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = default_req_actor::<Compile>(
                    MockCargoHandler::new(),
                    default_compile_params(TestCase::OneTarget),
                    TestCase::OneTarget,
                );

                req_actor.handle_cargo_information(CompilerMessageEnum(default_compiler_message(
                    DiagnosticLevel::Error,
                )));

                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.task.id" => RANDOM_TASK_ID,
                },@r#"
                {
                  "method": "build/publishDiagnostics",
                  "params": {
                    "buildTarget": {
                      "uri": "test_target"
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
                "#);
                no_more_msg(receiver_from_actor);
            }

            #[test]
            fn build_finished_simple() {
                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = default_req_actor::<Compile>(
                    MockCargoHandler::new(),
                    default_compile_params(TestCase::MultipleTargets),
                    TestCase::MultipleTargets,
                );

                req_actor.handle_cargo_information(BuildFinishedEnum(default_build_finished()));

                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                    ".params.data.time" => 0,
                    ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                },@r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "data": {
                      "errors": 0,
                      "originId": "test_origin_id",
                      "target": {
                        "uri": "test_target"
                      },
                      "time": 0,
                      "warnings": 0
                    },
                    "dataKind": "compile-report",
                    "eventTime": "timestamp",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": "[random_task_id]"
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                    ".params.data.time" => 0,
                    ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                },@r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "data": {
                      "errors": 0,
                      "originId": "test_origin_id",
                      "target": {
                        "uri": "test_target2"
                      },
                      "time": 0,
                      "warnings": 0
                    },
                    "dataKind": "compile-report",
                    "eventTime": "timestamp",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": "[random_task_id]"
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                },@r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "eventTime": "timestamp",
                    "message": "Finished compilation",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": [
                        "test_origin_id"
                      ]
                    }
                  }
                }
                "#);
                no_more_msg(receiver_from_actor);
            }

            #[test]
            fn build_finished_with_complex_compile_report() {
                // Checks if server counts warnings and error and produces a correct compile report.

                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = default_req_actor::<Compile>(
                    MockCargoHandler::new(),
                    default_compile_params(TestCase::OneTarget),
                    TestCase::OneTarget,
                );

                req_actor.handle_cargo_information(CompilerMessageEnum(default_compiler_message(
                    DiagnosticLevel::Error,
                )));
                req_actor.handle_cargo_information(CompilerMessageEnum(default_compiler_message(
                    DiagnosticLevel::Warning,
                )));

                let _ = receiver_from_actor.recv(); // publish diagnostic
                let _ = receiver_from_actor.recv(); // publish diagnostic

                req_actor.handle_cargo_information(BuildFinishedEnum(default_build_finished()));

                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                    ".params.data.time" => 0,
                    ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                },@r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "data": {
                      "errors": 1,
                      "originId": "test_origin_id",
                      "target": {
                        "uri": "test_target"
                      },
                      "time": 0,
                      "warnings": 1
                    },
                    "dataKind": "compile-report",
                    "eventTime": "timestamp",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": "[random_task_id]"
                    }
                  }
                }
                "#);
                assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.eventTime" => TIMESTAMP,
                    ".params.taskId.id" => RANDOM_TASK_ID,
                },@r#"
                {
                  "method": "build/taskFinish",
                  "params": {
                    "eventTime": "timestamp",
                    "message": "Finished compilation",
                    "status": 1,
                    "taskId": {
                      "id": "random_task_id",
                      "parents": [
                        "test_origin_id"
                      ]
                    }
                  }
                }
                "#);
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
                    .debuginfo(ArtifactDebuginfo::default())
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
        }
    }

    mod run_request_tests {
        use super::*;
        use crate::cargo_communication::cargo_types::event::CargoMessage::{
            CargoStderr, CargoStdout,
        };
        use bsp_types::requests::{Run, RunParams};
        use cargo_metadata::Message::TextLine;
        use serde_json::to_string;
        use std::io;

        const TEST_STDOUT: &str = "test_stdout";
        const TEST_STDERR: &str = "test_stderr";

        fn default_run_params() -> RunParams {
            RunParams {
                target: BuildTargetIdentifier {
                    uri: TEST_TARGET.into(),
                },
                origin_id: Some(TEST_ORIGIN_ID.into()),
                arguments: vec![TEST_ARGUMENTS.into()],
                data_kind: None,
                data: None,
            }
        }

        #[test]
        fn run_lifetime() {
            let mut mock_cargo_handle = MockCargoHandler::new();
            mock_cargo_handle
                .expect_join()
                .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
            let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();
            mock_cargo_handle
                .expect_receiver()
                .return_const(receiver_from_cargo);

            let TestEndpoints {
                req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Run>(
                mock_cargo_handle,
                default_run_params(),
                TestCase::OneTarget,
            );

            let _ = jod_thread::Builder::new()
                .spawn(move || req_actor.run())
                .expect("failed to spawn thread")
                .detach();

            let _ = receiver_from_actor.recv(); // main compilation task started
            let _ = receiver_from_actor.recv(); // target compilation task started

            sender_to_actor
                .send(CargoStdout(
                    to_string(&BuildFinishedEnum(default_build_finished())).unwrap(),
                ))
                .unwrap();

            let _ = receiver_from_actor.recv(); // main compilation task finished
            let _ = receiver_from_actor.recv(); // target compilation task finished

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.eventTime" => TIMESTAMP,
            }
            ,@r#"
            {
              "method": "build/taskStart",
              "params": {
                "eventTime": "timestamp",
                "message": "Started target execution",
                "taskId": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                }
              }
            }
            "#);

            drop(sender_to_actor);

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.eventTime" => TIMESTAMP,
            }
            ,@r#"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "message": "Finished target execution",
                "status": 1,
                "taskId": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                }
              }
            }
            "#);
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.eventTime" => TIMESTAMP,
            }
            ,@r#"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "status": 1,
                "taskId": {
                  "id": "test_origin_id"
                }
              }
            }
            "#);
            assert_json_snapshot!(receiver_from_actor.recv().unwrap()
            ,@r#"
            {
              "id": "test_req_id",
              "error": {
                "code": -32603,
                "message": "other error"
              }
            }
            "#);
            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn simple_stdout() {
            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Run>(
                MockCargoHandler::new(),
                default_run_params(),
                TestCase::OneTarget,
            );

            req_actor.handle_cargo_information(TextLine(TEST_STDOUT.to_string()));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                ".params.task.id" => RANDOM_TASK_ID,
            } ,@r#"
            {
              "method": "build/logMessage",
              "params": {
                "message": "test_stdout",
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
            "#);
            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn simple_stderr() {
            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Run>(
                MockCargoHandler::new(),
                default_run_params(),
                TestCase::OneTarget,
            );

            req_actor.handle_cargo_event(CargoStderr(TEST_STDERR.to_string()));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                ".params.task.id" => RANDOM_TASK_ID,
            } ,@r#"
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
            "#);
            no_more_msg(receiver_from_actor);
        }
    }

    #[cfg(test)]
    mod test_request_tests {
        use super::*;
        use crate::cargo_communication::cargo_types::event::CargoMessage::CargoStdout;
        use crate::cargo_communication::execution::cargo_types::test::TestEvent::Started;
        use crate::cargo_communication::execution::cargo_types::test::{
            SuiteEvent, SuiteResults, SuiteStarted, TestName, TestResult as TestResultEnum,
            TestType,
        };
        use crate::cargo_communication::utils::test_target_id;
        use bsp_types::requests::{Test, TestParams};
        use cargo_metadata::Message::TextLine;
        use crossbeam_channel::unbounded;
        use serde_json::to_string;
        use std::io;

        const TEST_NAME: &str = "test_name";

        fn default_test_params(test_case: TestCase) -> TestParams {
            let mut targets = vec![test_target_id(TEST_TARGET)];
            if let TestCase::MultipleTargets = test_case {
                targets.push(test_target_id(TEST_TARGET2));
            }
            TestParams {
                targets,
                origin_id: Some(TEST_ORIGIN_ID.into()),
                arguments: vec![TEST_ARGUMENTS.into()],
                data_kind: None,
                data: None,
            }
        }

        #[test]
        fn test_request_lifetime() {
            let mut mock_cargo_handle = MockCargoHandler::new();
            mock_cargo_handle
                .expect_join()
                .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
            let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();
            mock_cargo_handle
                .expect_receiver()
                .return_const(receiver_from_cargo);

            let TestEndpoints {
                req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Test>(
                mock_cargo_handle,
                default_test_params(TestCase::OneTarget),
                TestCase::OneTarget,
            );

            let _ = jod_thread::Builder::new()
                .spawn(move || req_actor.run())
                .expect("failed to spawn thread")
                .detach();

            let _ = receiver_from_actor.recv(); // main compilation task started
            let _ = receiver_from_actor.recv(); // target compilation task started

            sender_to_actor
                .send(CargoStdout(
                    to_string(&BuildFinishedEnum(default_build_finished())).unwrap(),
                ))
                .unwrap();

            let _ = receiver_from_actor.recv(); // main compilation task finished
            let _ = receiver_from_actor.recv(); // target compilation task finished

            // tests started
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.eventTime" => TIMESTAMP,
            }
            ,@r#"
            {
              "method": "build/taskStart",
              "params": {
                "eventTime": "timestamp",
                "message": "Started target testing",
                "taskId": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                }
              }
            }
            "#);

            drop(sender_to_actor);

            // tests finished
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.eventTime" => TIMESTAMP,
            }
            ,@r#"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "message": "Finished target testing",
                "status": 1,
                "taskId": {
                  "id": "random_task_id",
                  "parents": [
                    "test_origin_id"
                  ]
                }
              }
            }
            "#);

            // main task finished
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(),{
                ".params.eventTime" => TIMESTAMP,
            }
            ,@r#"
            {
              "method": "build/taskFinish",
              "params": {
                "eventTime": "timestamp",
                "status": 1,
                "taskId": {
                  "id": "test_origin_id"
                }
              }
            }
            "#);

            // response
            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), @r#"
            {
              "id": "test_req_id",
              "error": {
                "code": -32603,
                "message": "other error"
              }
            }
            "#);
            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn suite_started() {
            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Test>(
                MockCargoHandler::new(),
                default_test_params(TestCase::MultipleTargets),
                TestCase::MultipleTargets,
            );

            let suite_started = SuiteStarted { test_count: 1 };

            req_actor.handle_cargo_information(TextLine(
                to_string(&TestType::Suite(SuiteEvent::Started(suite_started))).unwrap(),
            ));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                ".params.eventTime" => TIMESTAMP,
            } ,@r#"
            {
              "method": "build/taskStart",
              "params": {
                "data": {
                  "target": {
                    "uri": "test_target2"
                  }
                },
                "dataKind": "test-task",
                "eventTime": "timestamp",
                "taskId": {
                  "id": "random_task_id",
                  "parents": "[random_task_id]"
                }
              }
            }
            "#);
            no_more_msg(receiver_from_actor);
        }

        fn default_suite_results() -> SuiteResults {
            SuiteResults {
                passed: 1,
                failed: 2,
                ignored: 3,
                measured: 4,
                filtered_out: 5,
                exec_time: 6.6,
            }
        }

        #[test]
        fn suite_results() {
            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Test>(
                MockCargoHandler::new(),
                default_test_params(TestCase::MultipleTargets),
                TestCase::MultipleTargets,
            );

            if let TaskState::Test(ref mut test_state) = req_actor.state.task_state {
                test_state.current_build_target = Some(BuildTargetIdentifier {
                    uri: TEST_TARGET2.to_string(),
                })
            }

            req_actor.handle_cargo_information(TextLine(
                to_string(&TestType::Suite(SuiteEvent::Ok(default_suite_results()))).unwrap(),
            ));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                ".params.eventTime" => TIMESTAMP,
            } ,@r#"
            {
              "method": "build/taskFinish",
              "params": {
                "data": {
                  "cancelled": 0,
                  "failed": 2,
                  "ignored": 3,
                  "passed": 1,
                  "skipped": 5,
                  "target": {
                    "uri": "test_target2"
                  },
                  "time": 6600
                },
                "dataKind": "test-report",
                "eventTime": "timestamp",
                "status": 1,
                "taskId": {
                  "id": "random_task_id",
                  "parents": "[random_task_id]"
                }
              }
            }
            "#);
            no_more_msg(receiver_from_actor);
        }

        #[test]
        fn test_started() {
            let TestEndpoints {
                mut req_actor,
                receiver_from_actor,
                _cancel_sender,
                ..
            } = default_req_actor::<Test>(
                MockCargoHandler::new(),
                default_test_params(TestCase::OneTarget),
                TestCase::OneTarget,
            );

            let test_started = Started(TestName {
                name: TEST_NAME.into(),
            });
            req_actor.handle_cargo_information(TextLine(
                to_string(&TestType::Test(test_started)).unwrap(),
            ));

            assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                ".params.taskId.id" => RANDOM_TASK_ID,
                ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                ".params.eventTime" => TIMESTAMP,
            } ,@r#"
            {
              "method": "build/taskStart",
              "params": {
                "data": {
                  "displayName": "test_name"
                },
                "dataKind": "test-start",
                "eventTime": "timestamp",
                "taskId": {
                  "id": "random_task_id",
                  "parents": "[random_task_id]"
                }
              }
            }
            "#);
            no_more_msg(receiver_from_actor);
        }

        mod test_finish_status {
            use super::*;
            use crate::cargo_communication::execution::cargo_types::test::TestEvent;
            use bsp_types::notifications::TestStatus;
            use insta::{allow_duplicates, dynamic_redaction};

            fn test_finish_status(passed_status: &TestType, expected_status: TestStatus) {
                let TestEndpoints {
                    mut req_actor,
                    receiver_from_actor,
                    _cancel_sender,
                    ..
                } = default_req_actor::<Test>(
                    MockCargoHandler::new(),
                    default_test_params(TestCase::OneTarget),
                    TestCase::OneTarget,
                );

                let test_started = Started(TestName {
                    name: TEST_NAME.into(),
                });
                req_actor.handle_cargo_information(TextLine(
                    to_string(&TestType::Test(test_started)).unwrap(),
                ));
                let _ = receiver_from_actor.recv().unwrap(); // test started message

                req_actor.handle_cargo_information(TextLine(to_string(passed_status).unwrap()));

                allow_duplicates!(assert_json_snapshot!(receiver_from_actor.recv().unwrap(), {
                    ".params.taskId.id" => RANDOM_TASK_ID,
                    ".params.taskId.parents" => format!("[{RANDOM_TASK_ID}]"),
                    ".params.eventTime" => TIMESTAMP,
                    ".params.data.status" => dynamic_redaction(move |value, _path| {
                        assert_eq!(value.as_u64().unwrap(), expected_status as u64);
                        "CORRECT_STATUS"
                    }),
                } ,@r#"
                  {
                    "method": "build/taskFinish",
                    "params": {
                      "data": {
                        "displayName": "test_name",
                        "status": "CORRECT_STATUS"
                      },
                      "dataKind": "test-finish",
                      "eventTime": "timestamp",
                      "status": 1,
                      "taskId": {
                        "id": "random_task_id",
                        "parents": "[random_task_id]"
                      }
                    }
                  }
                  "#));
                let _ = receiver_from_actor.recv().unwrap(); // test progress message
                no_more_msg(receiver_from_actor);
            }

            #[test]
            fn test_finished_ok() {
                test_finish_status(
                    &TestType::Test(TestEvent::Ok(default_test_result_enum())),
                    TestStatus::Passed,
                );
            }

            #[test]
            fn test_finish_failed() {
                test_finish_status(
                    &TestType::Test(TestEvent::Failed(default_test_result_enum())),
                    TestStatus::Failed,
                );
            }

            #[test]
            fn test_finish_ignored() {
                test_finish_status(
                    &TestType::Test(TestEvent::Ignored(default_test_result_enum())),
                    TestStatus::Ignored,
                );
            }

            #[test]
            fn test_finish_timeout() {
                test_finish_status(
                    &TestType::Test(TestEvent::Timeout(default_test_result_enum())),
                    TestStatus::Failed,
                );
            }

            fn default_test_result_enum() -> TestResultEnum {
                TestResultEnum {
                    name: TEST_NAME.to_string(),
                    stdout: None,
                }
            }
        }
    }
}
