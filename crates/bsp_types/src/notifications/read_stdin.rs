use crate::notifications::{Notification, TaskId};
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// Notification sent from the client to the server when the user wants to send
/// input to the stdin of the running target.
#[derive(Debug)]
pub enum OnRunReadStdin {}

impl Notification for OnRunReadStdin {
    type Params = ReadParams;
    const METHOD: &'static str = "run/readStdin";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadParams {
    /// The id of the request.
    pub origin_id: Identifier,
    /// Relevant only for test tasks.
    /// Allows to tell the client from which task the output is coming from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskId>,
    /// Message content can contain arbitrary bytes.
    /// They should be escaped as per [javascript encoding](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#using_special_characters_in_strings)
    pub message: String,
}
