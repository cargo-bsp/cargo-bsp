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
    pub(super) compile_state: CompileState,
    pub(super) execution_state: ExecutionState,
}

pub enum ExecutionState {
    Compile,
    Run(RunState),
    Test(TestState),
}

pub struct CompileState {
    pub(super) compile_task_id: TaskId,
    pub(super) compile_errors: i32,
    pub(super) compile_warnings: i32,
    pub(super) compile_start_time: i64,
}

pub struct RunState {
    pub(super) run_task_id: TaskId,
}

pub struct TestState {
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
            compile_state: CompileState {
                compile_task_id: TaskId {
                    id: TaskId::generate_random_id(),
                    parents: vec![root_task_id.id.clone()],
                },
                compile_errors: 0,
                compile_warnings: 0,
                compile_start_time: 0,
            },
            execution_state: RequestActorState::set_execution_state::<R>(root_task_id),
        }
    }

    fn set_execution_state<R: Request>(root_task_id: TaskId) -> ExecutionState {
        match R::METHOD {
            "buildTarget/run" => ExecutionState::Run(RunState {
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
                ExecutionState::Test(TestState {
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
            _ => ExecutionState::Compile,
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
                    self.finish_command();
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
        self.state.compile_state.compile_start_time = get_event_time().unwrap();
        // TODO change to actual BuildTargetIdentifier
        self.report_task_start(
            self.state.compile_state.compile_task_id.clone(),
            None,
            Some(TaskDataWithKind::CompileTask(CompileTaskData {
                target: Default::default(),
            })),
        );
    }

    fn finish_command(&mut self) {
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
        match &self.state.execution_state {
            ExecutionState::Compile => (),
            ExecutionState::Run(run_state) => self.report_task_finish(
                run_state.run_task_id.clone(),
                status_code.clone(),
                Some("Finished target execution".to_string()),
                None,
            ),
            ExecutionState::Test(test_state) => self.report_task_finish(
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
