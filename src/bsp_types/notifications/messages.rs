use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::bsp_types::notifications::{Notification, TaskId};

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
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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

/* Log message notification params */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum MessageType {
    #[default]
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::test_serialization;

    use super::*;

    #[test]
    fn show_message_method() {
        assert_eq!(ShowMessage::METHOD, "build/showMessage");
    }

    #[test]
    fn log_message_method() {
        assert_eq!(LogMessage::METHOD, "build/logMessage");
    }

    #[test]
    fn show_message_params() {
        let test_data = ShowMessageParams {
            message_type: MessageType::Error,
            task: Some(TaskId::default()),
            origin_id: Some("test_originId".to_string()),
            message: "test_message".to_string(),
        };

        test_serialization(
            &test_data,
            r#"{"type":1,"task":{"id":""},"originId":"test_originId","message":"test_message"}"#,
        );

        let mut modified_data = test_data.clone();
        modified_data.task = None;
        test_serialization(
            &modified_data,
            r#"{"type":1,"originId":"test_originId","message":"test_message"}"#,
        );
        modified_data = test_data.clone();
        modified_data.origin_id = None;
        test_serialization(
            &modified_data,
            r#"{"type":1,"task":{"id":""},"message":"test_message"}"#,
        );
    }

    #[test]
    fn log_message_params() {
        let test_data = LogMessageParams {
            message_type: MessageType::default(),
            task: Some(TaskId::default()),
            origin_id: Some("test_originId".to_string()),
            message: "test_message".to_string(),
        };

        test_serialization(
            &test_data,
            r#"{"type":1,"task":{"id":""},"originId":"test_originId","message":"test_message"}"#,
        );

        let mut modified_data = test_data.clone();
        modified_data.task = None;
        test_serialization(
            &modified_data,
            r#"{"type":1,"originId":"test_originId","message":"test_message"}"#,
        );
        modified_data = test_data.clone();
        modified_data.origin_id = None;
        test_serialization(
            &modified_data,
            r#"{"type":1,"task":{"id":""},"message":"test_message"}"#,
        );
    }

    #[test]
    fn message_type() {
        test_serialization(&MessageType::Error, r#"1"#);
        test_serialization(&MessageType::Warning, r#"2"#);
        test_serialization(&MessageType::Info, r#"3"#);
        test_serialization(&MessageType::Log, r#"4"#);
    }
}
