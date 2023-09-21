use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::{Notification, TaskId};
use crate::RequestId;

/// The show message notification is sent from a server to a client to ask the client to display a particular message in the user interface.
///
/// A build/showMessage notification is similar to LSP's window/showMessage, except for a few additions like id and originId.
#[derive(Debug)]
pub enum OnBuildShowMessage {}

impl Notification for OnBuildShowMessage {
    type Params = ShowMessageParams;
    const METHOD: &'static str = "build/showMessage";
}

/// The log message notification is sent from a server to a client to ask the client to log a particular message in its console.
///
/// A build/logMessage notification is similar to LSP's window/logMessage, except for a few additions like id and originId.
#[derive(Debug)]
pub enum OnBuildLogMessage {}

impl Notification for OnBuildLogMessage {
    type Params = LogMessageParams;
    const METHOD: &'static str = "build/logMessage";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowMessageParams {
    /// the message type.
    pub r#type: MessageType,
    /// The task id if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,
    /// The request id that originated this notification.
    /// The originId field helps clients know which request originated a notification in case several requests are handled by the
    /// client at the same time. It will only be populated if the client defined it in the request that triggered this notification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<RequestId>,
    /// The actual message.
    pub message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogMessageParams {
    /// the message type.
    pub r#type: MessageType,
    /// The task id if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,
    /// The request id that originated this notification.
    /// The originId field helps clients know which request originated a notification in case several requests are handled by the
    /// client at the same time. It will only be populated if the client defined it in the request that triggered this notification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<RequestId>,
    /// The actual message.
    pub message: String,
}

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum MessageType {
    #[default]
    /// An error message.
    Error = 1,
    /// A warning message.
    Warning = 2,
    /// An information message.
    Info = 3,
    /// A log message.
    Log = 4,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn show_message_method() {
        assert_eq!(OnBuildShowMessage::METHOD, "build/showMessage");
    }

    #[test]
    fn log_message_method() {
        assert_eq!(OnBuildLogMessage::METHOD, "build/logMessage");
    }

    #[test]
    fn show_message_params() {
        let test_data = ShowMessageParams {
            r#type: MessageType::Error,
            task: Some(TaskId::default()),
            origin_id: Some("test_originId".into()),
            message: "test_message".to_string(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "type": 1,
          "task": {
            "id": ""
          },
          "originId": "test_originId",
          "message": "test_message"
        }
        "#
        );
        assert_json_snapshot!(ShowMessageParams::default(),
            @r#"
        {
          "type": 1,
          "message": ""
        }
        "#
        );
    }

    #[test]
    fn log_message_params() {
        let test_data = LogMessageParams {
            r#type: MessageType::default(),
            task: Some(TaskId::default()),
            origin_id: Some("test_originId".into()),
            message: "test_message".to_string(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "type": 1,
          "task": {
            "id": ""
          },
          "originId": "test_originId",
          "message": "test_message"
        }
        "#
        );
        assert_json_snapshot!(LogMessageParams::default(),
            @r#"
        {
          "type": 1,
          "message": ""
        }
        "#
        );
    }

    #[test]
    fn message_type() {
        assert_json_snapshot!(MessageType::Error, @"1");
        assert_json_snapshot!(MessageType::Warning, @"2");
        assert_json_snapshot!(MessageType::Info, @"3");
        assert_json_snapshot!(MessageType::Log, @"4");
    }
}
