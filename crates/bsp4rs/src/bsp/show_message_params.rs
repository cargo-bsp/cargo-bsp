use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowMessageParams {
    /// the message type.
    pub r#type: MessageType,
    /// The task id if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,
    /// The request id that originated this notification.
    /// The originId field helps clients know which request originated a notification in case several requests are handled by the
    /// client at the same time. It will only be populated if the client defined it in the request that triggered this notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<OriginId>,
    /// The actual message.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

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
}
