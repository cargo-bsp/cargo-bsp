use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

use crossbeam_channel::{never, select, Receiver};
use log::info;
use serde_json::to_value;

use crate::bsp_types::notifications::{CompileTaskData, MessageType, TaskDataWithKind};
use crate::bsp_types::requests::Request;
use crate::bsp_types::StatusCode;
use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::request_actor_state::{RequestActorState, TaskState};
use crate::cargo_communication::utils::{generate_task_id, get_current_time};
use crate::communication::{ErrorCode, Message, ResponseError};
use crate::communication::{RequestId, Response};
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};

pub struct RequestActor<R>
where
    R: Request,
    R::Params: CreateCommand,
    R::Result: CargoResult,
{
    pub(super) sender: Box<dyn Fn(Message) + Send>,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<CargoHandle>,
    req_id: RequestId,
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

impl<R> RequestActor<R>
where
    R: Request,
    R::Params: CreateCommand,
    R::Result: CargoResult,
{
    pub fn new(
        sender: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> RequestActor<R> {
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

    pub fn run(mut self, cancel_receiver: Receiver<Event>, cargo_handle: CargoHandle) {
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
        self.send(Message::Response(self.create_response(res, &status_code)));
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

    fn create_response(
        &self,
        result: io::Result<ExitStatus>,
        status_code: &StatusCode,
    ) -> Response {
        Response {
            id: self.req_id.clone(),
            result: result.ok().map(|_| {
                to_value(R::Result::create_result(
                    self.params.origin_id(),
                    status_code.clone(),
                ))
                .unwrap()
            }),
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

    fn cancel_process(&self, cargo_handle: CargoHandle) {
        self.report_task_start(
            generate_task_id(&self.state.root_task_id),
            Some(format!("Start canceling request {}", self.req_id.clone())),
            None,
        );
        cargo_handle.cancel();
        self.report_task_finish(
            generate_task_id(&self.state.root_task_id),
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
        self.send(Message::Response(Response {
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
