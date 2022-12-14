use crate::bsp_types::MethodName;
use super::TaskId;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};


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
    pub origin_id: Option<String>,

    /** The actual message. */
    pub message: String,
}

impl MethodName for ShowMessageParams {
    fn get_method_name() -> &'static str {
        "build/showMessage"
    }
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
    pub origin_id: Option<String>,

    /** The actual message. */
    pub message: String,
}

impl MethodName for LogMessageParams {
    fn get_method_name() -> &'static str {
        "build/logMessage"
    }
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