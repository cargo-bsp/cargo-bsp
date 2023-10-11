use serde::{Deserialize, Serialize};

/// Represents the identifier of a BSP request.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OriginId(pub String);

impl OriginId {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for OriginId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for OriginId {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
}
