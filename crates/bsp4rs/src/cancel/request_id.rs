use serde::{Deserialize, Serialize};

/// Represents the identifier of a JsonRpc request id.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    I32(i32),
}
