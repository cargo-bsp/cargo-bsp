use crate::bsp_types::notifications::{
    get_event_time, LogMessage, LogMessageParams, MessageType, Notification as NotificationTrait,
    TaskDataWithKind, TaskFinish, TaskFinishParams, TaskId, TaskProgress, TaskProgressParams,
    TaskStart, TaskStartParams,
};
use crate::bsp_types::requests::{CreateCommand, CreateResult, Request};
use crate::bsp_types::StatusCode;
use crate::cargo_communication::request_actor::{
    CargoHandleTrait, CargoMessage, ExecutionState, RequestActor,
};
use crate::communication::{Message as RPCMessage, Notification};
use serde_json::to_value;

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand + CreateResult<R::Result>,
    C: CargoHandleTrait<CargoMessage>,
{
    pub(super) fn report_task_start(
        &self,
        task_id: TaskId,
        message: Option<String>,
        data: Option<TaskDataWithKind>,
    ) {
        self.send_notification::<TaskStart>(TaskStartParams {
            task_id,
            event_time: get_event_time(),
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
            event_time: get_event_time(),
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
            event_time: get_event_time(),
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
        let task_id = task_id.unwrap_or(match &self.state.execution_state {
            ExecutionState::Compile => self.state.root_task_id.clone(),
            ExecutionState::Run(run_state) => run_state.run_task_id.clone(),
            ExecutionState::Test(test_state) => test_state.test_task_id.clone(),
        });
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

    pub(super) fn send(&self, msg: RPCMessage) {
        (self.sender)(msg);
    }
}
