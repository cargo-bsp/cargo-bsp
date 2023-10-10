use crate::notifications::{Notification, TaskId};
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// Notification sent from the server to the client when the target being run or tested
/// prints something to stdout.
#[derive(Debug)]
pub enum OnRunPrintStdout {}

impl Notification for OnRunPrintStdout {
    type Params = PrintParams;
    const METHOD: &'static str = "run/printStdout";
}

/// Notification sent from the server to the client when the target being run or tested
/// prints something to stderr.
#[derive(Debug)]
pub enum OnRunPrintStderr {}

impl Notification for OnRunPrintStderr {
    type Params = PrintParams;
    const METHOD: &'static str = "run/printStderr";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrintParams {
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
