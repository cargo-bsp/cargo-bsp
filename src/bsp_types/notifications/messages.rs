use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::bsp_types::notifications::{Notification, TaskId};
use crate::bsp_types::OriginId;

#[derive(Debug)]
pub enum ShowMessage {}

impl Notification for ShowMessage {
    type Params = ShowMessageParams;
    const METHOD: &'static str = "build/showMessage";
}

#[derive(Debug)]
pub enum LogMessage {}

impl Notification for LogMessage {
    type Params = LogMessageParams;
    const METHOD: &'static str = "build/logMessage";
}

/* Show message notification */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ShowMessageParams {
    /** The message type. See {@link MessageType}. */
    #[serde(rename = "type")]
    pub message_type: MessageType,

    /** The task id if any. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,

    /** The request id that originated this notification. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<OriginId>,

    /** The actual message. */
    pub message: String,
}

/* Log message notification params */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageParams {
    /** The message type. See {@link MessageType}. */
    #[serde(rename = "type")]
    pub message_type: MessageType,

    /** The task id if any. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,

    /** The request id that originated this notification. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<OriginId>,

    /** The actual message. */
    pub message: String,
}

#[derive(Debug, Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum MessageType {
    #[default]
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
}
