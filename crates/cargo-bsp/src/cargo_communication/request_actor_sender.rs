//! Implementation of [`RequestActor`]. Sends responses and notifications back to
//! [`GlobalState`] that sends it back to the client.

use std::io;
use std::process::ExitStatus;

use bsp_server::{ErrorCode, Message, Notification, Response, ResponseError};
use serde_json::to_value;

use bsp_types::notifications::{
    LogMessage, LogMessageParams, MessageType, Notification as NotificationTrait, TaskDataWithKind,
    TaskFinish, TaskFinishParams, TaskId, TaskProgress, TaskProgressParams, TaskStart,
    TaskStartParams,
};
use bsp_types::requests::Request;
use bsp_types::StatusCode;

use crate::cargo_communication::cargo_types::cargo_command::CreateUnitGraphCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::CargoMessage;
use crate::cargo_communication::request_actor::{CargoHandler, RequestActor};
use crate::cargo_communication::utils::get_current_time;

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateUnitGraphCommand,
    R::Result: CargoResult,
    C: CargoHandler<CargoMessage>,
{
    pub(super) fn send_response(&self, command_result: io::Result<ExitStatus>) {
        self.send(Message::Response(Response {
            id: self.req_id.clone(),
            result: command_result.as_ref().ok().map(|exit_status| {
                to_value(R::Result::create_result(
                    self.params.origin_id(),
                    // If there is no exit code, process terminated by signal
                    exit_status.code().unwrap_or(143),
                ))
                .unwrap()
            }),
            error: command_result.as_ref().err().map(|e| ResponseError {
                code: ErrorCode::InternalError as i32,
                message: e.to_string(),
                data: None,
            }),
        }));
    }

    pub(super) fn send_cancel_response(&self) {
        self.report_task_finish(
            self.state.root_task_id.clone(),
            StatusCode::Cancelled,
            None,
            None,
        );
        let error = ResponseError {
            code: ErrorCode::RequestCanceled as i32,
            message: "canceled by client".to_string(),
            data: None,
        };
        self.send(
            Response {
                id: self.req_id.clone(),
                result: None,
                error: Some(error),
            }
            .into(),
        );
    }

    pub(super) fn report_root_task_start(&self) {
        self.report_task_start(self.state.root_task_id.clone(), None, None);
    }

    pub(super) fn report_task_start(
        &self,
        task_id: TaskId,
        message: Option<String>,
        data: Option<TaskDataWithKind>,
    ) {
        self.send_notification::<TaskStart>(TaskStartParams {
            task_id,
            event_time: Some(get_current_time()),
            message,
            data,
        });
    }

    pub(super) fn report_task_progress(
        &self,
        task_id: TaskId,
        message: Option<String>,
        total: Option<i64>,
        progress: Option<i64>,
        unit: Option<String>,
    ) {
        self.send_notification::<TaskProgress>(TaskProgressParams {
            task_id,
            event_time: Some(get_current_time()),
            message,
            total,
            progress,
            data: None,
            unit,
        });
    }

    pub(super) fn report_task_finish(
        &self,
        task_id: TaskId,
        status: StatusCode,
        message: Option<String>,
        data: Option<TaskDataWithKind>,
    ) {
        self.send_notification::<TaskFinish>(TaskFinishParams {
            task_id,
            event_time: Some(get_current_time()),
            message,
            status,
            data,
        });
    }

    pub(super) fn log_message(
        &self,
        message_type: MessageType,
        message: String,
        task_id: Option<TaskId>,
    ) {
        let task_id = task_id.unwrap_or(self.state.get_task_id());
        self.send_notification::<LogMessage>(LogMessageParams {
            message_type,
            task: Some(task_id),
            origin_id: self.params.origin_id(),
            message,
        });
    }

    pub(super) fn send_notification<T>(&self, notification: T::Params)
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

    pub(super) fn send(&self, msg: Message) {
        (self.sender)(msg);
    }
}
