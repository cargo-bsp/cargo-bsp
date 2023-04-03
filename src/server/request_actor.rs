#![warn(unused_lifetimes, semicolon_in_expressions_from_macros)]
#![allow(unused_variables)]

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crossbeam_channel::{never, select, unbounded, Receiver, Sender};
use lsp_types::DiagnosticSeverity;
use mockall::*;
use paths::AbsPath;
use serde::Serialize;
use serde_json::to_value;

use crate::bsp_types::{BuildTargetIdentifier, StatusCode};
// pub use cargo_metadata::diagnostic::{
//     Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
//     DiagnosticSpanMacroExpansion,
// };
// use cargo_metadata::Message;
use crate::bsp_types::cargo_output::Message;
use crate::bsp_types::mappings::test::{SuiteEvent, TestEvent, TestResult, TestType};
use crate::bsp_types::mappings::to_publish_diagnostics::create_diagnostics;
use crate::bsp_types::notifications::{
    get_event_time, CompileReportData, CompileTaskData, LogMessage, LogMessageParams, MessageType,
    Notification as NotificationTrait, PublishDiagnostics, TaskDataWithKind, TaskFinish,
    TaskFinishParams, TaskId, TaskProgress, TaskProgressParams, TaskStart, TaskStartParams,
    TestStartData, TestStatus, TestTaskData,
};
use crate::bsp_types::requests::{CreateCommand, CreateResult, Request};
use crate::communication::{ErrorCode, Message as RPCMessage, Notification, ResponseError};
use crate::communication::{RequestId, Response};
use crate::logger::log;
use crate::server::cargo_actor::CargoHandle;

pub enum Event {
    Cancel,
    CargoEvent(CargoMessage),
    CargoFinish,
}

#[derive(Debug)]
pub struct RequestHandle {
    #[allow(dead_code)]
    sender_to_cancel: Sender<Event>,
    _thread: jod_thread::JoinHandle,
}

impl RequestHandle {
    pub fn spawn<R>(
        sender_to_main: Box<dyn Fn(RPCMessage) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> RequestHandle
    where
        R: Request + 'static,
        R::Params: CreateCommand + Send + CreateResult<R::Result>,
    {
        let mut actor: RequestActor<R, CargoHandle> =
            RequestActor::new(sender_to_main, req_id, params, root_path);
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();
        let thread = jod_thread::Builder::new()
            .spawn(move || match actor.spawn_handle() {
                Ok(cargo_handle) => actor.run(receiver_to_cancel, cargo_handle),
                Err(err) => {
                    todo!()
                }
            })
            .expect("failed to spawn thread");
        RequestHandle {
            sender_to_cancel,
            _thread: thread,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(&self) {
        self.sender_to_cancel.send(Event::Cancel).unwrap();
    }
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
    sender: Box<dyn Fn(RPCMessage) + Send>,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<C>,
    req_id: RequestId,
    params: R::Params,
    root_path: PathBuf,
    state: RequestActorState,
}

struct RequestActorState {
    root_task_id: TaskId,
    compile_task_id: TaskId,
    compile_errors: i32,
    compile_warnings: i32,
    compile_start_time: i64,
    execution_task_id: TaskId,
    root_test_task_id: TaskId,
    single_test_task_ids: HashMap<String, TaskId>,
    started_tasks: HashSet<TaskId>,
}

impl RequestActorState {
    fn new(origin_id: Option<String>) -> RequestActorState {
        let root_task_id = TaskId {
            id: origin_id.unwrap_or(TaskId::generate_random_id()),
            parents: vec![],
        };
        RequestActorState {
            root_task_id: root_task_id.clone(),
            compile_task_id: TaskId {
                id: TaskId::generate_random_id(),
                parents: vec![root_task_id.id.clone()],
            },
            compile_errors: 0,
            compile_warnings: 0,
            compile_start_time: 0,
            execution_task_id: TaskId {
                id: TaskId::generate_random_id(),
                parents: vec![root_task_id.id.clone()],
            },
            root_test_task_id: TaskId {
                id: TaskId::generate_random_id(),
                parents: vec![root_task_id.id],
            },
            single_test_task_ids: HashMap::new(),
            started_tasks: HashSet::new(),
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
        log("Spawning a new request actor");
        RequestActor {
            sender,
            cargo_handle: None,
            req_id,
            state: RequestActorState::new(params.origin_id()),
            params,
            root_path: root_path.to_path_buf(),
        }
    }

    fn report_task_start(
        &mut self,
        task_id: TaskId,
        message: Option<String>,
        data: Option<TaskDataWithKind>,
    ) {
        self.send_notification::<TaskStart>(TaskStartParams {
            task_id: task_id.clone(),
            event_time: get_event_time(),
            message,
            data,
        });
        self.state.started_tasks.insert(task_id);
    }

    fn report_task_progress(&self, task_id: TaskId, message: Option<String>) {
        // TODO improve this
        self.send_notification::<TaskProgress>(TaskProgressParams {
            task_id,
            event_time: get_event_time(),
            message,
            total: None,
            progress: None,
            data: None,
            unit: None,
        });
    }

    fn report_task_finish(
        &mut self,
        task_id: TaskId,
        status: StatusCode,
        message: Option<String>,
        data: Option<TaskDataWithKind>,
    ) {
        self.send_notification::<TaskFinish>(TaskFinishParams {
            task_id: task_id.clone(),
            event_time: get_event_time(),
            message,
            status,
            data,
        });
        self.state.started_tasks.remove(&task_id);
    }

    fn log_message(&self, message_type: MessageType, message: String) {
        self.send_notification::<LogMessage>(LogMessageParams {
            message_type,
            task: Some(self.state.root_task_id.clone()),
            origin_id: self.params.origin_id(),
            message,
        })
    }

    fn send_notification<T>(&self, notification: T::Params)
    where
        T: NotificationTrait,
    {
        self.send(
            Notification {
                method: T::METHOD.to_string(),
                params: to_value(notification).unwrap(),
            }
            .into(),
        );
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
        log(format!("Created command: {:?}", command).as_str());
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => Ok(cargo_handle),
            Err(err) => {
                todo!()
            }
        }
    }

    pub fn run(mut self, cancel_receiver: Receiver<Event>, cargo_handle: C) {
        self.report_task_start(self.state.root_task_id.clone(), None, None);

        self.cargo_handle = Some(cargo_handle);

        self.start_compile_task();

        while let Some(event) = self.next_event(&cancel_receiver) {
            match event {
                Event::Cancel => {
                    self.cancel();
                    return;
                }
                Event::CargoFinish => {
                    let cargo_handle = self.cargo_handle.take().unwrap();
                    let res = cargo_handle.join();
                    let status_code = self.get_request_status_code(&res);
                    let resp = self.create_response(res, &status_code);
                    self.finish_execution_task(&status_code, R::METHOD.to_string());
                    self.report_task_finish(
                        self.state.root_task_id.clone(),
                        status_code,
                        None,
                        None,
                    );
                    self.send(RPCMessage::Response(resp));
                    return;
                }
                Event::CargoEvent(message) => {
                    // handle information and create notification based on that
                    match message {
                        CargoMessage::CargoStdout(stdout) => self.handle_cargo_information(stdout),
                        CargoMessage::CargoStderr(stderr) => {
                            self.log_message(MessageType::Error, stderr)
                        }
                    }
                }
            }
        }
    }

    fn start_compile_task(&mut self) {
        self.state.compile_start_time = get_event_time().unwrap();
        // TODO change to actual BuildTargetIdentifier
        self.report_task_start(
            self.state.compile_task_id.clone(),
            None,
            Some(TaskDataWithKind::CompileTask(CompileTaskData {
                target: Default::default(),
            })),
        );
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
            // TODO create error for response and finish any started tasks
            error: None,
        }
    }

    fn handle_cargo_information(&mut self, message: Message) {
        match message {
            Message::CompilerArtifact(msg) => {
                self.report_task_progress(
                    self.state.compile_task_id.clone(),
                    serde_json::to_string(&msg).ok(),
                );
            }
            Message::CompilerMessage(msg) => {
                let diagnostics = create_diagnostics(
                    &msg.message,
                    self.params.origin_id(),
                    // TODO change to actual BuildTargetIdentifier
                    &BuildTargetIdentifier {
                        uri: "".to_string(),
                    },
                    AbsPath::assert(&self.root_path),
                );
                diagnostics.into_iter().for_each(|diagnostic| {
                    // Count errors and warnings.
                    diagnostic.diagnostics.iter().for_each(|d| {
                        if let Some(severity) = d.severity {
                            match severity {
                                DiagnosticSeverity::ERROR => self.state.compile_errors += 1,
                                DiagnosticSeverity::WARNING => self.state.compile_warnings += 1,
                                _ => (),
                            }
                        }
                    });
                    self.send_notification::<PublishDiagnostics>(diagnostic)
                });
            }
            Message::BuildScriptExecuted(msg) => {
                self.report_task_progress(
                    self.state.compile_task_id.clone(),
                    serde_json::to_string(&msg).ok(),
                );
            }
            Message::BuildFinished(msg) => {
                let status_code = if msg.success {
                    StatusCode::Ok
                } else {
                    StatusCode::Error
                };
                let compile_report = TaskDataWithKind::CompileReport(CompileReportData {
                    // TODO change to actual BuildTargetIdentifier
                    target: Default::default(),
                    origin_id: self.params.origin_id(),
                    errors: self.state.compile_errors,
                    warnings: self.state.compile_warnings,
                    time: Some((get_event_time().unwrap() - self.state.compile_start_time) as i32),
                    no_op: None,
                });
                self.report_task_finish(
                    self.state.compile_task_id.clone(),
                    status_code,
                    None,
                    Some(compile_report),
                );
                self.start_execution_task(R::METHOD.to_string());
            }
            Message::TextLine(msg) => {
                let deserialized_message = serde_json::from_str::<TestType>(&msg);
                match deserialized_message {
                    // Message comes from running tests.
                    Ok(test_type) => self.handle_information_from_test(test_type),
                    // Message is a line from stdout.
                    Err(_) => self.log_message(MessageType::Log, msg),
                }
            }
        }
    }

    fn start_execution_task(&mut self, method: String) {
        if method.eq("buildTarget/run") {
            self.report_task_start(
                self.state.execution_task_id.clone(),
                Some("Started target execution".to_string()),
                None,
            );
        }
    }

    fn finish_execution_task(&mut self, status_code: &StatusCode, method: String) {
        if method.eq("buildTarget/run") {
            self.report_task_finish(
                self.state.execution_task_id.clone(),
                status_code.clone(),
                Some("Finished target execution".to_string()),
                None,
            );
        }
    }

    fn handle_information_from_test(&mut self, test_type: TestType) {
        // TODO change target to actual BuildTargetIdentifier
        match test_type {
            // Handle information about whole test suite.
            TestType::Suite(event) => match event {
                SuiteEvent::Started(_) => self.report_task_start(
                    self.state.root_test_task_id.clone(),
                    None,
                    Some(TaskDataWithKind::TestTask(TestTaskData {
                        target: Default::default(),
                    })),
                ),
                SuiteEvent::Ok(result) => self.report_task_finish(
                    self.state.root_test_task_id.clone(),
                    StatusCode::Ok,
                    None,
                    Some(result.to_test_report()),
                ),
                SuiteEvent::Failed(result) => self.report_task_finish(
                    self.state.root_test_task_id.clone(),
                    StatusCode::Error,
                    None,
                    Some(result.to_test_report()),
                ),
            },
            // Handle information about single test.
            TestType::Test(event) => match event {
                TestEvent::Started(started) => {
                    let test_task_id = TaskId {
                        id: TaskId::generate_random_id(),
                        parents: vec![self.state.root_test_task_id.id.clone()],
                    };
                    self.state
                        .single_test_task_ids
                        .insert(started.get_name(), test_task_id.clone());
                    self.report_task_start(
                        test_task_id,
                        None,
                        Some(TaskDataWithKind::TestStart(TestStartData {
                            display_name: started.get_name(),
                            location: None,
                        })),
                    );
                }
                TestEvent::Ok(result) => self.finish_single_test(result, TestStatus::Passed),
                TestEvent::Failed(result) => self.finish_single_test(result, TestStatus::Failed),
                TestEvent::Ignored(result) => self.finish_single_test(result, TestStatus::Ignored),
                TestEvent::Timeout(result) => self.finish_single_test(result, TestStatus::Failed),
            },
        }
    }

    fn finish_single_test(&mut self, test_result: TestResult, status: TestStatus) {
        let task_id = self
            .state
            .single_test_task_ids
            .remove(&test_result.get_name());
        if let Some(id) = task_id {
            self.report_task_finish(
                id,
                StatusCode::Ok,
                None,
                Some(TaskDataWithKind::TestFinish(
                    test_result.map_to_test_notification(status),
                )),
            );
        }
    }

    fn cancel(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            self.cancel_process(cargo_handle);
            self.cancel_tasks_and_request();
        } else {
            todo!()
        }
    }

    fn cancel_process(&mut self, cargo_handle: C) {
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

    fn cancel_tasks_and_request(&self) {
        // TODO cancel other started tasks
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

    fn send(&self, msg: RPCMessage) {
        (self.sender)(msg);
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

    use lsp_types::{DiagnosticSeverity, NumberOrString, Position, Range};
    use serde_json::to_string;

    use crate::bsp_types::cargo_output::{
        Artifact, ArtifactProfile, BuildFinished, BuildScript, CompilerMessage, Diagnostic,
        DiagnosticCode, DiagnosticLevel, DiagnosticSpan, DiagnosticSpanLine, Edition,
        Message::CompilerArtifact, PackageId, Target,
    };
    // use cargo_metadata::Message::CompilerArtifact;
    // use cargo_metadata::{Artifact, ArtifactProfile, Edition, Target};
    use crate::bsp_types::cargo_output::Message::{
        BuildFinished as BuildFinishedEnum, BuildScriptExecuted,
        CompilerMessage as CompilerMessageEnum,
    };
    use crate::bsp_types::notifications::{
        CompileTaskData, Diagnostic as LSPDiagnostic, PublishDiagnosticsParams, TaskDataWithKind,
        TaskFinish, TaskFinishParams, TaskId, TaskProgress, TaskProgressParams, TaskStart,
        TaskStartParams,
    };
    use crate::bsp_types::requests::{Compile, CompileParams, CompileResult};
    use crate::bsp_types::TextDocumentIdentifier;
    use crate::communication::{ErrorCode, Message, Notification, Response};
    use crate::server::request_actor::CargoMessage::CargoStdout;

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
                "test_req_id".into(),
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
            "test_req_id".into(),
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
            "test_req_id".into(),
            ErrorCode::RequestCanceled as i32,
            "".into(),
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

        let compiler_artifact = Artifact {
            package_id: PackageId {
                repr: "test_package_id".into(),
            },
            target: Target {
                name: "test_target_name".to_string(),
                kind: vec!["test_kind".into()],
                crate_types: vec!["test_crate_type".into()],
                required_features: vec!["test_required_feature".into()],
                src_path: "test_src_path".into(),
                edition: Edition::E2021,
                doctest: false,
                test: false,
                doc: false,
            },
            profile: ArtifactProfile {
                opt_level: "test_opt_level".into(),
                debuginfo: Some(0),
                debug_assertions: false,
                overflow_checks: false,
                test: false,
            },
            features: vec!["test_feature".into()],
            filenames: vec!["test_filename".into()],
            executable: Some("test_executable".into()),
            fresh: false,
        };
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

        let build_script_output = BuildScript {
            package_id: PackageId {
                repr: "test_package_id".into(),
            },
            linked_libs: vec!["test_linked_lib".into()],
            linked_paths: vec!["test_linked_path".into()],
            cfgs: vec!["test_cfg".into()],
            env: vec![("test_env".into(), "test_env".into())],
            out_dir: "test_out_dir".into(),
        };
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

    #[test]
    fn compiler_message_with_one_publish_diagnostic() {
        #[allow(unused_mut)]
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        let (recv_to_main, send_from_cargo, send_to_cancel) = init_test(mock_cargo_handle);

        let _ = recv_to_main.recv(); // main task started
        let _ = recv_to_main.recv(); // compilation task started

        let compiler_mess = CompilerMessage {
            package_id: PackageId {
                repr: "test_package_id".into(),
            },
            target: Target {
                name: "test_target_name".into(),
                kind: vec!["test_kind".into()],
                crate_types: vec!["test_crate_type".into()],
                required_features: vec!["test_required_feature".into()],
                src_path: "test_src_path".into(),
                edition: Edition::E2021,
                doctest: false,
                test: false,
                doc: false,
            },
            message: Diagnostic {
                message: "test_message".into(),
                code: Some(DiagnosticCode {
                    code: "test_code".to_string(),
                    explanation: Some("test_explanation".into()),
                }),
                level: DiagnosticLevel::Warning,
                spans: vec![DiagnosticSpan {
                    file_name: "test_filename".into(),
                    byte_start: 1,
                    byte_end: 2,
                    line_start: 3,
                    line_end: 4,
                    column_start: 5,
                    column_end: 6,
                    is_primary: true, // TODO czemu musi byc primary w to_publish_diagnostic.rs?
                    text: vec![DiagnosticSpanLine {
                        text: "test_text".into(),
                        highlight_start: 7,
                        highlight_end: 8,
                    }],
                    label: Some("test_label".into()),
                    suggested_replacement: Some("test_suggested_replacement".into()),
                    suggestion_applicability: None,
                    expansion: None,
                }],
                children: vec![],
                rendered: None,
            },
        };
        send_from_cargo
            .send(CargoStdout(CompilerMessageEnum(compiler_mess)))
            .unwrap();
        let proper_publish_diagnostic = Notification::new(
            PublishDiagnostics::METHOD.to_string(),
            PublishDiagnosticsParams {
                text_document: TextDocumentIdentifier {
                    uri: "file::///test_root_path/test_filename".into(),
                },
                build_target: "test_target_name".into(),
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
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String("test_code".to_string())),
                    code_description: None,
                    source: Some("cargo".into()),
                    message: "test_text".to_string(),
                    related_information: None,
                    tags: None,
                    data: None,
                }],
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

        let build_finished = BuildFinished { success: true };
        send_from_cargo
            .send(CargoStdout(BuildFinishedEnum(build_finished)))
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

        let compiler_message_warning = CompilerMessage {
            package_id: PackageId {
                repr: "test_package_id".into(),
            },
            target: Target {
                name: "test_target_name".into(),
                kind: vec!["test_kind".into()],
                crate_types: vec!["test_crate_type".into()],
                required_features: vec!["test_required_feature".into()],
                src_path: "test_src_path".into(),
                edition: Edition::E2021,
                doctest: false,
                test: false,
                doc: false,
            },
            message: Diagnostic {
                message: "".to_string(),
                code: None,
                level: DiagnosticLevel::Warning,
                spans: vec![DiagnosticSpan {
                    file_name: "test_filename".into(),
                    byte_start: 1,
                    byte_end: 2,
                    line_start: 3,
                    line_end: 4,
                    column_start: 5,
                    column_end: 6,
                    is_primary: true,
                    text: vec![DiagnosticSpanLine {
                        text: "test_text".into(),
                        highlight_start: 7,
                        highlight_end: 8,
                    }],
                    label: Some("test_label".into()),
                    suggested_replacement: Some("test_suggested_replacement".into()),
                    suggestion_applicability: None,
                    expansion: None,
                }],
                children: vec![],
                rendered: None,
            },
        };
        let compiler_message_error = CompilerMessage {
            package_id: PackageId {
                repr: "test_package_id".into(),
            },
            target: Target {
                name: "test_target_name".into(),
                kind: vec!["test_kind".into()],
                crate_types: vec!["test_crate_type".into()],
                required_features: vec!["test_required_feature".into()],
                src_path: "test_src_path".into(),
                edition: Edition::E2021,
                doctest: false,
                test: false,
                doc: false,
            },
            message: Diagnostic {
                message: "".to_string(),
                code: None,
                level: DiagnosticLevel::Error,
                spans: vec![DiagnosticSpan {
                    file_name: "test_filename".into(),
                    byte_start: 1,
                    byte_end: 2,
                    line_start: 3,
                    line_end: 4,
                    column_start: 5,
                    column_end: 6,
                    is_primary: true,
                    text: vec![DiagnosticSpanLine {
                        text: "test_text".into(),
                        highlight_start: 7,
                        highlight_end: 8,
                    }],
                    label: Some("test_label".into()),
                    suggested_replacement: Some("test_suggested_replacement".into()),
                    suggestion_applicability: None,
                    expansion: None,
                }],
                children: vec![],
                rendered: None,
            },
        };
        send_from_cargo
            .send(CargoStdout(CompilerMessageEnum(compiler_message_warning)))
            .unwrap();
        send_from_cargo
            .send(CargoStdout(CompilerMessageEnum(compiler_message_error)))
            .unwrap();

        // TODO: sprawdzic czy powinno byc tak ze dostajemy publish diagnostic i dodajemy warningi tylko gdy jest cos w spanie
        let _ = recv_to_main.recv(); // publish diagnostic
        let _ = recv_to_main.recv(); // publish diagnostic

        let build_finished = BuildFinished { success: true };
        send_from_cargo
            .send(CargoStdout(BuildFinishedEnum(build_finished)))
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

// #[cfg(test)]
// pub mod run_request_tests {
//     use super::*;
//     use crate::bsp_types::notifications::{
//         CompileTaskData, TaskDataWithKind, TaskFinish, TaskFinishParams, TaskId, TaskProgress,
//         TaskProgressParams, TaskStart, TaskStartParams,
//     };
//     use crate::bsp_types::requests::{
//         Compile, CompileParams, CompileResult, Run, RunParams, Test, TestParams,
//     };
//     use crate::communication::{ErrorCode, Message, Notification, Response};
//     use crate::server::request_actor::CargoMessage::CargoStdout;
//     use crate::server::request_actor::Event::CargoFinish;
//     use cargo_metadata::Message::CompilerArtifact;
//     use std::os::unix::prelude::ExitStatusExt;
//
//     fn get_run_actor(
//         sender_to_main: Sender<RPCMessage>,
//     ) -> RequestActor<Run, MockCargoHandleTrait<CargoMessage>> {
//         RequestActor::new(
//             Box::new(move |msg| sender_to_main.send(msg).unwrap()),
//             "test_req_id".into(),
//             RunParams {
//                 target: "test_target".into(),
//                 origin_id: Some("test_origin_id".into()),
//                 arguments: vec!["test_arguments".into()],
//                 data_kind: Some("test_data_kind".into()),
//                 data: Some("test_data".into()),
//             },
//             Path::new("test_root_path"),
//         )
//     }
// }
//
// #[cfg(test)]
// pub mod test_request_tests {
//     use super::*;
//     use crate::bsp_types::notifications::{
//         CompileTaskData, TaskDataWithKind, TaskFinish, TaskFinishParams, TaskId, TaskProgress,
//         TaskProgressParams, TaskStart, TaskStartParams,
//     };
//     use crate::bsp_types::requests::{
//         Compile, CompileParams, CompileResult, Run, RunParams, Test, TestParams,
//     };
//     use crate::communication::{ErrorCode, Message, Notification, Response};
//     use crate::server::request_actor::CargoMessage::CargoStdout;
//     use crate::server::request_actor::Event::CargoFinish;
//     use cargo_metadata::Message::CompilerArtifact;
//     use std::os::unix::prelude::ExitStatusExt;
//
//     fn get_test_actor(
//         sender_to_main: Sender<RPCMessage>,
//     ) -> RequestActor<Test, MockCargoHandleTrait<CargoMessage>> {
//         RequestActor::new(
//             Box::new(move |msg| sender_to_main.send(msg).unwrap()),
//             "test_req_id".into(),
//             TestParams {
//                 targets: vec!["test_target".into()],
//                 origin_id: Some("test_origin_id".into()),
//                 arguments: vec!["test_arguments".into()],
//                 data_kind: Some("test_data_kind".into()),
//                 data: Some("test_data".into()),
//             },
//             Path::new("test_root_path"),
//         )
//     }
// }
