use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use bsp_server::Message;
use bsp_server::{RequestId, Response};
use crossbeam_channel::{never, select, Receiver};
use log::{info, warn};
use mockall::*;
use serde_json::to_value;

use crate::bsp_types::notifications::{CompileTaskData, MessageType, TaskDataWithKind};
use crate::bsp_types::requests::Request;
use crate::bsp_types::StatusCode;
use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::request_actor_state::{RequestActorState, TaskState};
use crate::cargo_communication::utils::get_current_time;
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};

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

#[cfg(not(test))]
fn get_request_status_code(result: &io::Result<ExitStatus>) -> StatusCode {
    match result {
        Ok(exit_status) if exit_status.success() => StatusCode::Ok,
        _ => StatusCode::Error,
    }
}

#[cfg(test)]
fn get_request_status_code(_: &io::Result<ExitStatus>) -> StatusCode {
    StatusCode::Ok
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
pub trait CargoHandleTrait<T> {
    fn receiver(&self) -> &Receiver<T>;

    fn cancel(self);

    fn join(self) -> io::Result<ExitStatus>;
}

#[cfg(test)]
pub mod compile_request_tests {
    use std::collections::HashSet;
    use std::fmt::Debug;
    use std::hash::Hash;
    use std::time::Duration;

    use crate::bsp_types::notifications::{
        CompileReportData, CompileTaskData, Diagnostic as LspDiagnostic,
        Notification as NotificationTrait, PublishDiagnostics, PublishDiagnosticsParams,
        TaskDataWithKind, TaskFinish, TaskFinishParams, TaskId, TaskProgress, TaskProgressParams,
        TaskStart, TaskStartParams,
    };
    use crate::bsp_types::requests::{Compile, CompileParams, CompileResult};
    use crate::bsp_types::{BuildTargetIdentifier, TextDocumentIdentifier};
    use crate::cargo_communication::cargo_types::event::CargoMessage::CargoStdout;
    use bsp_server::{ErrorCode, Message, Notification, Response};
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
    use lsp_types::{
        DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString, Position, Range,
    };
    use serde_json::to_string;
    use url::Url;

    use super::*;

    fn init_test(
        mut mock_cargo_handle: MockCargoHandleTrait<CargoMessage>,
    ) -> (Receiver<Message>, Sender<CargoMessage>, Sender<Event>) {
        let (sender_to_main, receiver_from_actor) = unbounded::<Message>();
        let (sender_to_actor, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();

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
        let _ = jod_thread::Builder::new()
            .spawn(move || req_actor.run(cancel_receiver, mock_cargo_handle))
            .expect("failed to spawn thread")
            .detach();

        (receiver_from_actor, sender_to_actor, cancel_sender)
    }

    fn assert_is_last_msg<T: PartialEq + Debug>(receiver: Receiver<T>, msg: T) {
        assert_eq!(receiver.recv().unwrap(), msg);
        assert!(receiver.recv_timeout(Duration::from_secs(1)).is_err());
    }

    #[test]
    fn simple_compile() {
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        // There is no robust way to return ExitStatus hence we return Error. In consequence there
        // are specific implementations of create_response() and get_request_status_code() for tests.
        mock_cargo_handle
            .expect_join()
            .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
        let (recv_from_actor, send_to_actor, _cancel_sender) = init_test(mock_cargo_handle);

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
            recv_from_actor.recv().unwrap(),
            Message::Notification(proper_notif_start_main_task)
        );
        assert_eq!(
            recv_from_actor.recv().unwrap(),
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

        drop(send_to_actor);

        assert_eq!(
            recv_from_actor.recv().unwrap(),
            Message::Notification(proper_notif_finish_main_task)
        );
        assert_is_last_msg(recv_from_actor, Message::Response(proper_response));
    }

    #[test]
    fn simple_cancel() {
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle.expect_cancel().return_const(());
        // There is no robust way to return ExitStatus hence we return Error. In consequence there
        // are specific implementations of create_response() and get_request_status_code() for tests.
        mock_cargo_handle
            .expect_join()
            .returning(|| Err(io::Error::from(io::ErrorKind::Other)));
        let (recv_from_actor, _send_to_actor, cancel_sender) = init_test(mock_cargo_handle);

        let _ = recv_from_actor.recv(); // main task started
        let _ = recv_from_actor.recv(); // compilation task started

        cancel_sender.send(Event::Cancel).unwrap_or_else(|e| {
            panic!("failed to send cancel event: {}", e);
        });

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
            recv_from_actor.recv().unwrap(),
            Message::Notification(proper_notif_finish_main_task)
        );
        assert_is_last_msg(recv_from_actor, Message::Response(proper_response));
    }

    #[test]
    fn compiler_artifact() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_from_actor, send_to_actor, _cancel_sender) = init_test(mock_cargo_handle);

        let _ = recv_from_actor.recv(); // main task started
        let _ = recv_from_actor.recv(); // compilation task started

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

        send_to_actor
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
        assert_is_last_msg(
            recv_from_actor,
            Message::Notification(proper_notif_task_progress),
        );
    }

    #[test]
    fn build_script_out() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_from_actor, send_to_actor, _cancel_sender) = init_test(mock_cargo_handle);

        let _ = recv_from_actor.recv(); // main task started
        let _ = recv_from_actor.recv(); // compilation task started

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

        send_to_actor
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
        assert_is_last_msg(
            recv_from_actor,
            Message::Notification(proper_notif_task_progress),
        );
    }

    fn eq_unordered_vec<T>(a: &[T], b: &[T]) -> bool
    where
        T: Eq + Hash,
    {
        let a: HashSet<_> = a.iter().collect();
        let b: HashSet<_> = b.iter().collect();

        a == b
    }

    #[test]
    fn compiler_message() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_from_actor, send_to_actor, _cancel_sender) = init_test(mock_cargo_handle);

        let _ = recv_from_actor.recv(); // main task started
        let _ = recv_from_actor.recv(); // compilation task started

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
                    .spans(vec![
                        DiagnosticSpanBuilder::default()
                            .file_name("test_filename1".to_string())
                            .byte_start(0_u32)
                            .byte_end(0_u32)
                            .line_start(1_usize)
                            .line_end(2_usize)
                            .column_start(3_usize)
                            .column_end(4_usize)
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
                            .unwrap(),
                        DiagnosticSpanBuilder::default()
                            .file_name("test_filename2".to_string())
                            .byte_start(0_u32)
                            .byte_end(0_u32)
                            .line_start(1_usize)
                            .line_end(2_usize)
                            .column_start(3_usize)
                            .column_end(4_usize)
                            .is_primary(false)
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
                            .unwrap(),
                    ])
                    .children(vec![DiagnosticBuilder::default()
                        .message("test_child_message".to_string())
                        .code(None)
                        .level(DiagnosticLevel::Help)
                        .spans(vec![DiagnosticSpanBuilder::default()
                            .file_name("test_filename1".to_string())
                            .byte_start(0_u32)
                            .byte_end(0_u32)
                            .line_start(5_usize)
                            .line_end(6_usize)
                            .column_start(7_usize)
                            .column_end(8_usize)
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
                        .unwrap()])
                    .rendered(Some("test_rendered".to_string()))
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        send_to_actor
            .send(CargoStdout(CompilerMessage(compiler_mess)))
            .unwrap();

        let proper_publish_diagnostic_first = Message::Notification(Notification::new(
            PublishDiagnostics::METHOD.to_string(),
            PublishDiagnosticsParams {
                text_document: TextDocumentIdentifier {
                    uri: "file:///test_root_path/test_filename1".into(),
                },
                build_target: "".into(),
                //TODO change to "test_target_name" later
                origin_id: Some("test_origin_id".into()),
                diagnostics: vec![
                    LspDiagnostic {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 2,
                            },
                            end: Position {
                                line: 1,
                                character: 3,
                            },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("test_code".to_string())),
                        code_description: None,
                        source: Some("cargo".into()),
                        message: "test_message\ntest_label".to_string(),
                        related_information: Some(vec![
                            DiagnosticRelatedInformation {
                                location: Location {
                                    uri: Url::parse("file:///test_root_path/test_filename2")
                                        .unwrap(),
                                    range: Range {
                                        start: Position {
                                            line: 0,
                                            character: 2,
                                        },
                                        end: Position {
                                            line: 1,
                                            character: 3,
                                        },
                                    },
                                },
                                message: "test_label".to_string(),
                            },
                            DiagnosticRelatedInformation {
                                location: Location {
                                    uri: Url::parse("file:///test_root_path/test_filename1")
                                        .unwrap(),
                                    range: Range {
                                        start: Position {
                                            line: 4,
                                            character: 6,
                                        },
                                        end: Position {
                                            line: 5,
                                            character: 7,
                                        },
                                    },
                                },
                                message: "test_child_message".to_string(),
                            },
                        ]),
                        tags: None,
                        data: None,
                    },
                    LspDiagnostic {
                        range: Range {
                            start: Position {
                                line: 4,
                                character: 6,
                            },
                            end: Position {
                                line: 5,
                                character: 7,
                            },
                        },
                        severity: Some(DiagnosticSeverity::HINT),
                        code: Some(NumberOrString::String("test_code".to_string())),
                        code_description: None,
                        source: Some("cargo".into()),
                        message: "test_child_message".to_string(),
                        related_information: Some(vec![DiagnosticRelatedInformation {
                            location: Location {
                                uri: Url::parse("file:///test_root_path/test_filename1").unwrap(),
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 2,
                                    },
                                    end: Position {
                                        line: 1,
                                        character: 3,
                                    },
                                },
                            },
                            message: "original diagnostic".to_string(),
                        }]),
                        tags: None,
                        data: None,
                    },
                ],
                reset: false,
            },
        ));

        let proper_publish_diagnostic_second = Message::Notification(Notification::new(
            PublishDiagnostics::METHOD.to_string(),
            PublishDiagnosticsParams {
                text_document: TextDocumentIdentifier {
                    uri: "file:///test_root_path/test_filename2".into(),
                },
                build_target: "".into(),
                //TODO change to "test_target_name" later
                origin_id: Some("test_origin_id".into()),
                diagnostics: vec![LspDiagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 2,
                        },
                        end: Position {
                            line: 1,
                            character: 3,
                        },
                    },
                    severity: Some(DiagnosticSeverity::HINT),
                    code: Some(NumberOrString::String("test_code".to_string())),
                    code_description: None,
                    source: Some("cargo".into()),
                    message: "test_label".to_string(),
                    related_information: Some(vec![DiagnosticRelatedInformation {
                        location: Location {
                            uri: Url::parse("file:///test_root_path/test_filename1").unwrap(),
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 2,
                                },
                                end: Position {
                                    line: 1,
                                    character: 3,
                                },
                            },
                        },
                        message: "original diagnostic".to_string(),
                    }]),
                    tags: None,
                    data: None,
                }],
                reset: false,
            },
        ));

        assert!(eq_unordered_vec(
            &[
                to_string(&proper_publish_diagnostic_first).unwrap(),
                to_string(&proper_publish_diagnostic_second).unwrap(),
            ],
            &[
                to_string(&recv_from_actor.recv().unwrap()).unwrap(),
                to_string(&recv_from_actor.recv().unwrap()).unwrap()
            ]
        ));
        assert!(recv_from_actor
            .recv_timeout(Duration::from_millis(100))
            .is_err());
    }

    #[test]
    fn build_finished_simple() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_from_actor, send_to_actor, _cancel_sender) = init_test(mock_cargo_handle);

        let _ = recv_from_actor.recv(); // main task started
        let _ = recv_from_actor.recv(); // compilation task started

        let build_finished = BuildFinishedBuilder::default()
            .success(true)
            .build()
            .unwrap();
        send_to_actor
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
        assert_is_last_msg(recv_from_actor, Message::Notification(proper_task_finished));
    }

    #[test]
    fn build_finished_with_complex_compile_report() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_from_actor, send_to_actor, _cancel_sender) = init_test(mock_cargo_handle);

        let _ = recv_from_actor.recv(); // main task started
        let _ = recv_from_actor.recv(); // compilation task started

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

        send_to_actor
            .send(CargoStdout(CompilerMessage(compiler_message_warning)))
            .unwrap();
        send_to_actor
            .send(CargoStdout(CompilerMessage(compiler_message_error)))
            .unwrap();

        let _ = recv_from_actor.recv(); // publish diagnostic
        let _ = recv_from_actor.recv(); // publish diagnostic

        let build_finished = BuildFinishedBuilder::default()
            .success(true)
            .build()
            .unwrap();

        send_to_actor
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
        assert_is_last_msg(recv_from_actor, Message::Notification(proper_task_finished));
    }
}
