use bsp_server::{Message, Notification};
use serde_json::to_value;

use crate::bsp_types::notifications::{
    LogMessage, LogMessageParams, MessageType, Notification as NotificationTrait, TaskDataWithKind,
    TaskFinish, TaskFinishParams, TaskId, TaskProgress, TaskProgressParams, TaskStart,
    TaskStartParams,
};
use crate::bsp_types::requests::Request;
use crate::bsp_types::StatusCode;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::request_actor::RequestActor;
use crate::cargo_communication::utils::get_current_time;

impl<R> RequestActor<R>
where
    R: Request,
    R::Params: CreateCommand,
    R::Result: CargoResult,
{
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
