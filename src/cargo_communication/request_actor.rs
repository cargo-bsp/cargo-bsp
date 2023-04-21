#![warn(unused_lifetimes, semicolon_in_expressions_from_macros)]
#![allow(unused_variables)]

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crossbeam_channel::{never, select, Receiver};
use log::info;
use mockall::*;
use serde::Serialize;
use serde_json::to_value;

use crate::bsp_types::notifications::{
    get_event_time, CompileTaskData, MessageType, TaskDataWithKind, TaskId,
};
use crate::bsp_types::requests::{CreateCommand, CreateResult, Request};
use crate::bsp_types::StatusCode;
use crate::cargo_communication::cargo_actor::CargoHandle;
use crate::communication::{ErrorCode, Message as RPCMessage, ResponseError};
use crate::communication::{RequestId, Response};
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use cargo_metadata::Message;

pub enum Event {
    Cancel,
    CargoEvent(CargoMessage),
    CargoFinish,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum CargoMessage {
    CargoStdout(Message),
    CargoStderr(String),
}

pub struct RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand + CreateResult<R::Result>,
    C: CargoHandleTrait<CargoMessage>,
{
    pub(super) sender: Box<dyn Fn(RPCMessage) + Send>,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<C>,
    req_id: RequestId,
    pub(super) params: R::Params,
    pub(super) root_path: PathBuf,
    pub(super) state: RequestActorState,
}

pub struct RequestActorState {
    pub(super) root_task_id: TaskId,
    pub(super) compile_task_state: CompileTaskState,
    pub(super) task_state: TaskState,
}

pub enum TaskState {
    Compile,
    Run(RunTaskState),
    Test(TestTaskState),
}

pub struct CompileTaskState {
    pub(super) compile_task_id: TaskId,
    pub(super) compile_errors: i32,
    pub(super) compile_warnings: i32,
    pub(super) compile_start_time: i64,
}

pub struct RunTaskState {
    pub(super) run_task_id: TaskId,
}

pub struct TestTaskState {
    pub(super) test_task_id: TaskId,
    pub(super) suite_test_task_id: TaskId,
    pub(super) suite_task_progress: SuiteTaskProgress,
    pub(super) single_test_task_ids: HashMap<String, TaskId>,
}

pub struct SuiteTaskProgress {
    pub(super) progress: i64,
    pub(super) total: i64,
}

impl RequestActorState {
    fn new<R: Request>(origin_id: Option<String>) -> RequestActorState {
        let root_task_id = TaskId {
            id: origin_id.unwrap_or(TaskId::generate_random_id()),
            parents: vec![],
        };
        RequestActorState {
            root_task_id: root_task_id.clone(),
            compile_task_state: CompileTaskState {
                compile_task_id: TaskId {
                    id: TaskId::generate_random_id(),
                    parents: vec![root_task_id.id.clone()],
                },
                compile_errors: 0,
                compile_warnings: 0,
                compile_start_time: 0,
            },
            task_state: RequestActorState::set_task_state::<R>(root_task_id),
        }
    }

    fn set_task_state<R: Request>(root_task_id: TaskId) -> TaskState {
        match R::METHOD {
            "buildTarget/run" => TaskState::Run(RunTaskState {
                run_task_id: TaskId {
                    id: TaskId::generate_random_id(),
                    parents: vec![root_task_id.id],
                },
            }),
            "buildTarget/test" => {
                let test_task_id = TaskId {
                    id: TaskId::generate_random_id(),
                    parents: vec![root_task_id.id],
                };
                TaskState::Test(TestTaskState {
                    suite_test_task_id: TaskId {
                        id: Default::default(),
                        parents: vec![test_task_id.id.clone()],
                    },
                    suite_task_progress: SuiteTaskProgress {
                        progress: 0,
                        total: 0,
                    },
                    test_task_id,
                    single_test_task_ids: HashMap::new(),
                })
            }
            _ => TaskState::Compile,
        }
    }
}

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand + CreateResult<R::Result>,
    C: CargoHandleTrait<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(RPCMessage) + Send>,
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

    pub fn spawn_handle(&mut self) -> Result<CargoHandle, String> {
        let command = self.params.create_command(self.root_path.clone());
        info!("Created command: {:?}", command);
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => Ok(cargo_handle),
            Err(err) => {
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
                        CargoMessage::CargoStdout(stdout) => self.handle_cargo_information(stdout),
                        CargoMessage::CargoStderr(stderr) => {
                            self.log_message(MessageType::Error, stderr, None)
                        }
                    }
                }
            }
        }
    }

    fn start_compile_task(&mut self) {
        self.state.compile_task_state.compile_start_time = get_event_time().unwrap();
        // TODO change to actual BuildTargetIdentifier
        self.report_task_start(
            self.state.compile_task_state.compile_task_id.clone(),
            None,
            Some(TaskDataWithKind::CompileTask(CompileTaskData {
                target: Default::default(),
            })),
        );
    }

    fn finish_request(&mut self) {
        let res = self.cargo_handle.take().unwrap().join();
        let status_code = self.get_request_status_code(&res);

        self.finish_execution_task(&status_code);
        self.report_task_finish(
            self.state.root_task_id.clone(),
            status_code.clone(),
            None,
            None,
        );
        self.send(RPCMessage::Response(
            self.create_response(res, &status_code),
        ));
    }

    fn finish_execution_task(&self, status_code: &StatusCode) {
        match &self.state.task_state {
            TaskState::Compile => (),
            TaskState::Run(run_state) => self.report_task_finish(
                run_state.run_task_id.clone(),
                status_code.clone(),
                Some("Finished target execution".to_string()),
                None,
            ),
            TaskState::Test(test_state) => self.report_task_finish(
                test_state.test_task_id.clone(),
                status_code.clone(),
                Some("Finished target testing".to_string()),
                None,
            ),
        }
    }

    fn get_request_status_code(&self, result: &io::Result<ExitStatus>) -> StatusCode {
        match result {
            Ok(exit_status) => {
                if exit_status.success() {
                    StatusCode::Ok
                } else {
                    StatusCode::Error
                }
            }
            Err(_) => StatusCode::Error,
        }
    }

    fn create_response(
        &self,
        result: io::Result<ExitStatus>,
        status_code: &StatusCode,
    ) -> Response {
        Response {
            id: self.req_id.clone(),
            result: result
                .ok()
                .map(|_| to_value(self.params.create_result(status_code.clone())).unwrap()),
            // TODO create error for response
            error: None,
        }
    }

    fn cancel(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            self.cancel_process(cargo_handle);
            self.cancel_task_and_request();
        } else {
            todo!("trzeba wyslac ze Task sie nie powiodl")
        }
    }

    fn cancel_process(&self, cargo_handle: C) {
        self.report_task_start(
            TaskId {
                id: TaskId::generate_random_id(),
                parents: vec![self.state.root_task_id.id.clone()],
            },
            Some(format!("Start canceling request {}", self.req_id.clone())),
            None,
        );
        cargo_handle.cancel();
        self.report_task_finish(
            TaskId {
                id: TaskId::generate_random_id(),
                parents: vec![self.state.root_task_id.id.clone()],
            },
            StatusCode::Ok,
            Some(format!("Finish canceling request {}", self.req_id.clone())),
            None,
        );
    }

    fn cancel_task_and_request(&self) {
        self.report_task_finish(
            self.state.root_task_id.clone(),
            StatusCode::Cancelled,
            None,
            None,
        );
        self.send(RPCMessage::Response(Response {
            id: self.req_id.clone(),
            result: None,
            error: Some(ResponseError {
                code: ErrorCode::RequestCanceled as i32,
                message: format!("Request {} canceled", self.req_id.clone()),
                data: None,
            }),
        }));
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
    use std::os::unix::prelude::ExitStatusExt;

    use crate::bsp_types::notifications::{
        CompileReportData, CompileTaskData, Diagnostic as LSPDiagnostic,
        Notification as NotificationTrait, PublishDiagnostics, PublishDiagnosticsParams,
        TaskDataWithKind, TaskFinish, TaskFinishParams, TaskId, TaskProgress, TaskProgressParams,
        TaskStart, TaskStartParams,
    };
    use crate::bsp_types::requests::{Compile, CompileParams, CompileResult};
    use crate::bsp_types::{BuildTargetIdentifier, TextDocumentIdentifier};
    use crate::cargo_communication::request_actor::CargoMessage::CargoStdout;
    use crate::communication::{ErrorCode, Message, Notification, Response};
    use cargo_metadata::diagnostic::{
        DiagnosticBuilder, DiagnosticCodeBuilder, DiagnosticSpanBuilder, DiagnosticSpanLineBuilder,
    };
    use cargo_metadata::Message::{
        BuildFinished, BuildScriptExecuted, CompilerArtifact, CompilerMessage,
    };
    use cargo_metadata::{
        ArtifactBuilder, ArtifactProfileBuilder, BuildFinishedBuilder, BuildScriptBuilder,
        CompilerMessageBuilder, Edition, PackageId, TargetBuilder,
    };
    use crossbeam_channel::{unbounded, Sender};
    use lsp_types::{DiagnosticSeverity, NumberOrString, Position, Range};
    use serde_json::to_string;

    use super::*;

    fn init_test(
        mut mock_cargo_handle: MockCargoHandleTrait<CargoMessage>,
    ) -> (Receiver<RPCMessage>, Sender<CargoMessage>, Sender<Event>) {
        let (sender_to_main, receiver_to_main) = unbounded::<RPCMessage>();
        let (sender_from_cargo, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();

        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);

        let req_actor: RequestActor<Compile, MockCargoHandleTrait<CargoMessage>> =
            RequestActor::new(
                Box::new(move |msg| sender_to_main.send(msg).unwrap()),
                "test_req_id".to_string().into(),
                CompileParams {
                    targets: vec!["test_target".into()],
                    origin_id: Some("test_origin_id".into()),
                    arguments: vec!["test_arguments".into()],
                },
                Path::new("/test_root_path"),
            );
        let thread = jod_thread::Builder::new()
            .spawn(move || req_actor.run(receiver_to_cancel, mock_cargo_handle))
            .expect("failed to spawn thread")
            .detach();

        (receiver_to_main, sender_from_cargo, sender_to_cancel)
    }

    #[test]
    fn simple_compile() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle
            .expect_join()
            .returning(|| Ok(ExitStatus::from_raw(0)));
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let proper_notif_start_main_task = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId {
                    id: "test_origin_id".to_string(),
                    parents: vec![],
                },
                event_time: Some(1),
                message: None,
                data: None,
            },
        );
        let proper_notif_start_compile_task = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId {
                    id: "random_task_id".to_string(),
                    parents: vec!["test_origin_id".to_string()],
                },
                event_time: Some(1),
                message: None,
                data: Some(TaskDataWithKind::CompileTask(CompileTaskData {
                    //TODO change to "test_target" later
                    target: Default::default(),
                })),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_start_main_task)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_start_compile_task)
        );

        let proper_notif_finish_main_task = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "test_origin_id".to_string(),
                    parents: vec![],
                },
                event_time: Some(1),
                message: None,
                data: None,
                status: StatusCode::Ok,
            },
        );
        let proper_response = Response::new_ok(
            "test_req_id".to_string().into(),
            CompileResult {
                origin_id: "test_origin_id".to_string().into(),
                status_code: StatusCode::Ok,
                data_kind: None,
                data: None,
            },
        );

        drop(send_from_cargo);

        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_finish_main_task)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Response(proper_response)
        );
    }

    #[test]
    fn simple_cancel() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle.expect_cancel().return_const(());
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        send_to_cancel.send(Event::Cancel).unwrap();

        let proper_notif_start_cancel = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId {
                    id: "random_task_id".to_string(),
                    parents: vec!["test_origin_id".to_string()],
                },
                event_time: Some(1),
                message: Some("Start canceling request \"test_req_id\"".to_string()),
                data: None,
            },
        );
        let proper_notif_finish_cancel = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".to_string(),
                    parents: vec!["test_origin_id".to_string()],
                },
                event_time: Some(1),
                message: Some("Finish canceling request \"test_req_id\"".to_string()),
                data: None,
                status: StatusCode::Ok,
            },
        );
        let proper_notif_finish_main_task = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "test_origin_id".to_string(),
                    parents: vec![],
                },
                event_time: Some(1),
                message: None,
                data: None,
                status: StatusCode::Cancelled,
            },
        );
        let proper_response = Response::new_err(
            "test_req_id".to_string().into(),
            ErrorCode::RequestCanceled as i32,
            "Request \"test_req_id\" canceled".into(),
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_start_cancel)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_finish_cancel)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_finish_main_task)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Response(proper_response)
        );
    }

    #[test]
    fn compiler_artifact() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let compiler_artifact = ArtifactBuilder::default()
            .package_id(PackageId {
                repr: "test_package_id".into(),
            })
            .manifest_path("test_manifest_path".to_string())
            .target(
                TargetBuilder::default()
                    .name("test_target_name".to_string())
                    .kind(vec!["test_kind".into()])
                    .crate_types(vec!["test_crate_type".into()])
                    .required_features(vec!["test_required_feature".into()])
                    .src_path("test_src_path".to_string())
                    .edition(Edition::E2021)
                    .doctest(false)
                    .test(false)
                    .doc(false)
                    .build()
                    .unwrap(),
            )
            .profile(
                ArtifactProfileBuilder::default()
                    .opt_level("test_opt_level".to_string())
                    .debuginfo(Some(0))
                    .debug_assertions(false)
                    .overflow_checks(false)
                    .test(false)
                    .build()
                    .unwrap(),
            )
            .features(vec!["test_feature".into()])
            .filenames(vec!["test_filename".into()])
            .executable(Some("test_executable".into()))
            .fresh(false)
            .build()
            .unwrap();

        send_from_cargo
            .send(CargoStdout(CompilerArtifact(compiler_artifact.clone())))
            .unwrap();

        let proper_notif_task_progress = Notification::new(
            TaskProgress::METHOD.to_string(),
            TaskProgressParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: Some(to_string(&compiler_artifact).unwrap()),
                total: None,
                progress: None,
                unit: None,
                data: None,
            },
        );

        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_task_progress)
        );
    }

    #[test]
    fn build_script_out() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let build_script_output = BuildScriptBuilder::default()
            .package_id(PackageId {
                repr: "test_package_id".to_string(),
            })
            .linked_libs(vec!["test_linked_lib".into()])
            .linked_paths(vec!["test_linked_path".into()])
            .cfgs(vec!["test_cfg".into()])
            .env(vec![("test_env".into(), "test_env".into())])
            .out_dir("test_out_dir".to_string())
            .build()
            .unwrap();

        send_from_cargo
            .send(CargoStdout(BuildScriptExecuted(
                build_script_output.clone(),
            )))
            .unwrap();

        let proper_notif_task_progress = Notification::new(
            TaskProgress::METHOD.to_string(),
            TaskProgressParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: Some(to_string(&build_script_output).unwrap()),
                total: None,
                progress: None,
                unit: None,
                data: None,
            },
        );

        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif_task_progress)
        );
    }

    #[ignore]
    #[test]
    fn compiler_message_with_one_publish_diagnostic() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let compiler_mess = CompilerMessageBuilder::default()
            .package_id(PackageId {
                repr: "test_package_id".to_string(),
            })
            .target(
                TargetBuilder::default()
                    .name("test_target_name".to_string())
                    .kind(vec!["test_kind".into()])
                    .crate_types(vec!["test_crate_type".into()])
                    .required_features(vec!["test_required_feature".into()])
                    .src_path("test_src_path".to_string())
                    .edition(Edition::E2021)
                    .doctest(false)
                    .test(false)
                    .doc(false)
                    .build()
                    .unwrap(),
            )
            .message(
                DiagnosticBuilder::default()
                    .message("test_message".to_string())
                    .code(Some(
                        DiagnosticCodeBuilder::default()
                            .code("test_code".to_string())
                            .explanation(Some("test_explanation".to_string()))
                            .build()
                            .unwrap(),
                    ))
                    .level(DiagnosticLevel::Error)
                    .spans(vec![DiagnosticSpanBuilder::default()
                        .file_name("test_file_name".to_string())
                        .byte_start(0_u32)
                        .byte_end(0_u32)
                        .line_start(1_usize)
                        .line_end(2_usize)
                        .column_start(3_usize)
                        .column_end(4_usize)
                        .is_primary(true) // TODO czemu musi byc primary w to_publish_diagnostic.rs?
                        .text(vec![DiagnosticSpanLineBuilder::default()
                            .text("test_text".to_string())
                            .highlight_start(0_usize)
                            .highlight_end(0_usize)
                            .build()
                            .unwrap()])
                        .label(Some("test_label".to_string()))
                        .suggested_replacement(None)
                        .suggestion_applicability(None)
                        .expansion(None)
                        .build()
                        .unwrap()])
                    .children(vec![DiagnosticBuilder::default()
                        .message("test_child_message".to_string())
                        .code(None)
                        .level(DiagnosticLevel::Help)
                        .spans(vec![])
                        .children(vec![])
                        .rendered(None)
                        .build()
                        .unwrap()])
                    .rendered(Some("test_rendered".to_string()))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        send_from_cargo
            .send(CargoStdout(CompilerMessage(compiler_mess)))
            .unwrap();

        let proper_publish_diagnostic = Notification::new(
            PublishDiagnostics::METHOD.to_string(),
            PublishDiagnosticsParams {
                text_document: TextDocumentIdentifier {
                    uri: "file::///test_root_path/test_filename".into(),
                },
                build_target: "".into(),
                //TODO change to "test_target_name" later
                origin_id: Some("test_origin_id".into()),
                diagnostics: vec![LSPDiagnostic {
                    range: Range {
                        start: Position {
                            line: 2,
                            character: 4,
                        },
                        end: Position {
                            line: 3,
                            character: 5,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: Some(NumberOrString::String("test_code".to_string())),
                    code_description: None,
                    source: Some("cargo".into()),
                    message: "test_message".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }],
                // TODO add children to list of diagnostics (?)
                reset: false,
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_publish_diagnostic)
        );
    }

    #[test]
    fn compile_message_with_many_publish_diagnostics() {}

    #[test]
    fn build_finished_simple() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let build_finished = BuildFinishedBuilder::default()
            .success(true)
            .build()
            .unwrap();
        send_from_cargo
            .send(CargoStdout(BuildFinished(build_finished)))
            .unwrap();
        let proper_task_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: Some(TaskDataWithKind::CompileReport(CompileReportData {
                    // TODO do poprawy
                    target: BuildTargetIdentifier { uri: "".into() },
                    origin_id: Some("test_origin_id".into()),
                    errors: 0,
                    warnings: 0,
                    time: Some(0),
                    no_op: None,
                })),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_task_finished)
        );
    }

    #[test]
    fn build_finished_with_complexed_compile_report() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let compiler_message_warning = CompilerMessageBuilder::default()
            .package_id(PackageId {
                repr: "test_package_id".to_string(),
            })
            .target(
                TargetBuilder::default()
                    .name("test_target_name".to_string())
                    .kind(vec!["test_kind".to_string()])
                    .crate_types(vec!["test_crate_type".to_string()])
                    .required_features(vec!["test_required_feature".to_string()])
                    .src_path("test_src_path".to_string())
                    .edition(Edition::E2021)
                    .doctest(false)
                    .test(false)
                    .doc(false)
                    .build()
                    .unwrap(),
            )
            .message(
                DiagnosticBuilder::default()
                    .message("".to_string())
                    .code(None)
                    .level(DiagnosticLevel::Warning)
                    .spans(vec![DiagnosticSpanBuilder::default()
                        .file_name("test_filename".to_string())
                        .byte_start(0_u32)
                        .byte_end(0_u32)
                        .line_start(0_usize)
                        .line_end(0_usize)
                        .column_start(0_usize)
                        .column_end(0_usize)
                        .is_primary(true)
                        .text(vec![DiagnosticSpanLineBuilder::default()
                            .text("test_text".to_string())
                            .highlight_start(0_usize)
                            .highlight_end(0_usize)
                            .build()
                            .unwrap()])
                        .label(Some("test_label".to_string()))
                        .suggested_replacement(None)
                        .suggestion_applicability(None)
                        .expansion(None)
                        .build()
                        .unwrap()])
                    .children(vec![])
                    .rendered(None)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let compiler_message_error = CompilerMessageBuilder::default()
            .package_id(PackageId {
                repr: "test_package_id".to_string(),
            })
            .target(
                TargetBuilder::default()
                    .name("test_target_name".to_string())
                    .kind(vec!["test_kind".to_string()])
                    .crate_types(vec!["test_crate_type".to_string()])
                    .required_features(vec!["test_required_feature".to_string()])
                    .src_path("test_src_path".to_string())
                    .edition(Edition::E2021)
                    .doctest(false)
                    .test(false)
                    .doc(false)
                    .build()
                    .unwrap(),
            )
            .message(
                DiagnosticBuilder::default()
                    .message("".to_string())
                    .code(None)
                    .level(DiagnosticLevel::Error)
                    .spans(vec![DiagnosticSpanBuilder::default()
                        .file_name("test_filename".to_string())
                        .byte_start(0_u32)
                        .byte_end(0_u32)
                        .line_start(0_usize)
                        .line_end(0_usize)
                        .column_start(0_usize)
                        .column_end(0_usize)
                        .is_primary(true)
                        .text(vec![DiagnosticSpanLineBuilder::default()
                            .text("test_text".to_string())
                            .highlight_start(0_usize)
                            .highlight_end(0_usize)
                            .build()
                            .unwrap()])
                        .label(Some("test_label".to_string()))
                        .suggested_replacement(None)
                        .suggestion_applicability(None)
                        .expansion(None)
                        .build()
                        .unwrap()])
                    .children(vec![])
                    .rendered(None)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        send_from_cargo
            .send(CargoStdout(CompilerMessage(compiler_message_warning)))
            .unwrap();
        send_from_cargo
            .send(CargoStdout(CompilerMessage(compiler_message_error)))
            .unwrap();

        // TODO: sprawdzic czy powinno byc tak ze dostajemy publish diagnostic i dodajemy warningi tylko gdy jest cos w spanie
        let _ = recv_to_main.recv(); // publish diagnostic
        let _ = recv_to_main.recv(); // publish diagnostic

        let build_finished = BuildFinishedBuilder::default()
            .success(true)
            .build()
            .unwrap();

        send_from_cargo
            .send(CargoStdout(BuildFinished(build_finished)))
            .unwrap();
        let proper_task_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: Some(TaskDataWithKind::CompileReport(CompileReportData {
                    // TODO do poprawy
                    target: BuildTargetIdentifier { uri: "".into() },
                    origin_id: Some("test_origin_id".into()),
                    errors: 1,
                    warnings: 1,
                    time: Some(0),
                    no_op: None,
                })),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_task_finished)
        );
    }
}

#[cfg(test)]
pub mod run_request_tests {
    use crate::bsp_types::notifications::{
        CompileReportData, LogMessage, LogMessageParams, Notification as NotificationTrait,
        TaskFinish, TaskFinishParams, TaskStart, TaskStartParams,
    };
    use crate::bsp_types::requests::{Run, RunParams, RunResult};
    use crate::bsp_types::BuildTargetIdentifier;
    use crate::cargo_communication::request_actor::CargoMessage::{CargoStderr, CargoStdout};
    use crate::communication::{Message, Notification, Response};
    use cargo_metadata::BuildFinishedBuilder;
    use cargo_metadata::Message::{BuildFinished, TextLine};
    use crossbeam_channel::{unbounded, Sender};
    use std::os::unix::process::ExitStatusExt;

    use super::*;

    fn init_test(
        mut mock_cargo_handle: MockCargoHandleTrait<CargoMessage>,
    ) -> (Receiver<RPCMessage>, Sender<CargoMessage>, Sender<Event>) {
        let (sender_to_main, receiver_to_main) = unbounded::<RPCMessage>();
        let (sender_from_cargo, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();

        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);

        let req_actor: RequestActor<Run, MockCargoHandleTrait<CargoMessage>> = RequestActor::new(
            Box::new(move |msg| sender_to_main.send(msg).unwrap()),
            "test_req_id".to_string().into(),
            RunParams {
                target: "test_target".into(),
                origin_id: Some("test_origin_id".into()),
                arguments: vec!["--test_argument".into()],
                data_kind: Some("test_data_kind".into()),
                data: Some("test_data".into()),
            },
            Path::new("/test_root_path"),
        );
        let thread = jod_thread::Builder::new()
            .spawn(move || req_actor.run(receiver_to_cancel, mock_cargo_handle))
            .expect("failed to spawn thread")
            .detach();

        (receiver_to_main, sender_from_cargo, sender_to_cancel)
    }

    #[test]
    fn simple_run() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle
            .expect_join()
            .returning(|| Ok(ExitStatus::from_raw(0)));
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started
        send_from_cargo
            .send(CargoStdout(BuildFinished(
                BuildFinishedBuilder::default()
                    .success(true)
                    .build()
                    .unwrap(),
            )))
            .unwrap();
        let compile_subtask_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: Some(TaskDataWithKind::CompileReport(CompileReportData {
                    // TODO do poprawy
                    target: BuildTargetIdentifier { uri: "".into() },
                    origin_id: Some("test_origin_id".into()),
                    errors: 0,
                    warnings: 0,
                    time: Some(0),
                    no_op: None,
                })),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(compile_subtask_finished)
        );
        let run_subtask_started = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: Some("Started target execution".into()),
                data: None,
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(run_subtask_started)
        );
        drop(send_from_cargo);

        let run_subtask_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: Some("Finished target execution".into()),
                status: StatusCode::Ok,
                data: None,
            },
        );
        let run_task_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "test_origin_id".into(),
                    parents: vec![],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: None,
            },
        );
        let proper_resp = Response::new_ok(
            "test_req_id".to_string().into(),
            RunResult {
                origin_id: Some("test_origin_id".into()),
                status_code: StatusCode::Ok,
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(run_subtask_finished)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(run_task_finished)
        );
        assert_eq!(recv_to_main.recv().unwrap(), Message::Response(proper_resp));
    }

    #[test]
    fn simple_stdout() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let text_line = TextLine("test_text_line".to_string());
        send_from_cargo.send(CargoStdout(text_line)).unwrap();

        let proper_notif = Notification::new(
            LogMessage::METHOD.to_string(),
            LogMessageParams {
                message_type: MessageType::Log,
                message: "test_text_line".to_string(),
                origin_id: Some("test_origin_id".into()),
                task: Some(TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                }),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif)
        );
    }

    #[test]
    fn simple_stderr() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let stderr = CargoStderr("test_stderr".to_string());
        send_from_cargo.send(stderr).unwrap();

        let proper_notif = Notification::new(
            LogMessage::METHOD.to_string(),
            LogMessageParams {
                message_type: MessageType::Error,
                message: "test_stderr".to_string(),
                origin_id: Some("test_origin_id".into()),
                task: Some(TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                }),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(proper_notif)
        );
    }
}
#[cfg(test)]
pub mod test_request_tests {
    #[allow(unused_imports)]
    use crate::bsp_types::mappings::test::{
        SuiteEvent, SuiteResults, SuiteStarted, TestEvent, TestName, TestResult as TestResultEnum,
        TestType,
    };
    use crate::bsp_types::notifications::{
        CompileReportData, Notification as NotificationTrait, TaskFinish, TaskFinishParams,
        TaskStart, TaskStartParams,
    };
    use crate::bsp_types::requests::{Test, TestParams, TestResult};
    use crate::bsp_types::BuildTargetIdentifier;
    #[allow(unused_imports)]
    use crate::cargo_communication::request_actor::CargoMessage::{CargoStderr, CargoStdout};
    use crate::communication::{Message, Notification, Response};
    use cargo_metadata::BuildFinishedBuilder;
    use cargo_metadata::Message::{BuildFinished, TextLine};
    use crossbeam_channel::{unbounded, Sender};
    use serde_json::to_string;
    use std::os::unix::process::ExitStatusExt;

    use super::*;

    fn init_test(
        mut mock_cargo_handle: MockCargoHandleTrait<CargoMessage>,
    ) -> (Receiver<RPCMessage>, Sender<CargoMessage>, Sender<Event>) {
        let (sender_to_main, receiver_to_main) = unbounded::<RPCMessage>();
        let (sender_from_cargo, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();

        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);

        let req_actor: RequestActor<Test, MockCargoHandleTrait<CargoMessage>> = RequestActor::new(
            Box::new(move |msg| sender_to_main.send(msg).unwrap()),
            "test_req_id".to_string().into(),
            TestParams {
                targets: vec!["test_target".into()],
                origin_id: Some("test_origin_id".into()),
                arguments: vec!["--test_argument".into()],
                data_kind: Some("test_data_kind".into()),
                data: Some("test_data".into()),
            },
            Path::new("/test_root_path"),
        );
        let thread = jod_thread::Builder::new()
            .spawn(move || req_actor.run(receiver_to_cancel, mock_cargo_handle))
            .expect("failed to spawn thread")
            .detach();

        (receiver_to_main, sender_from_cargo, sender_to_cancel)
    }

    #[ignore]
    #[test]
    fn simple_test() {
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle
            .expect_join()
            .returning(|| Ok(ExitStatus::from_raw(0)));
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started
        send_from_cargo
            .send(CargoStdout(BuildFinished(
                BuildFinishedBuilder::default()
                    .success(true)
                    .build()
                    .unwrap(),
            )))
            .unwrap();
        let compile_subtask_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: Some(TaskDataWithKind::CompileReport(CompileReportData {
                    // TODO do poprawy
                    target: BuildTargetIdentifier { uri: "".into() },
                    origin_id: Some("test_origin_id".into()),
                    errors: 0,
                    warnings: 0,
                    time: Some(0),
                    no_op: None,
                })),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(compile_subtask_finished)
        );
        let test_subtask_started = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: Some("Started target testing".into()),
                data: None,
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(test_subtask_started)
        );
        drop(send_from_cargo);
        let test_subtask_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["test_origin_id".into()],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: None,
            },
        );
        let test_task_finished = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId {
                    id: "test_origin_id".into(),
                    parents: vec![],
                },
                event_time: Some(1),
                message: None,
                status: StatusCode::Ok,
                data: None,
            },
        );
        let test_response = Response::new_ok(
            "test_req_id".to_string().into(),
            TestResult {
                origin_id: Some("test_origin_id".into()),
                status_code: StatusCode::Ok,
                data_kind: Some("test_data_kind".into()),
                data: Some("test_data".into()),
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(test_subtask_finished)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(test_task_finished)
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Response(test_response)
        );
    }

    #[ignore]
    #[test]
    fn suite_started() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let suite_started = SuiteStarted { test_count: 1 };
        send_from_cargo
            .send(CargoStdout(TextLine(
                to_string(&TestType::Suite(SuiteEvent::Started(suite_started))).unwrap(),
            )))
            .unwrap();

        let suite_task_started = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId {
                    id: "random_task_id".into(),
                    parents: vec!["random_task_id".into()],
                },
                event_time: Some(1),
                message: Some("Started test suite".into()),
                data: None,
            },
        );
        assert_eq!(
            recv_to_main.recv().unwrap(),
            Message::Notification(suite_task_started)
        );
    }
}
