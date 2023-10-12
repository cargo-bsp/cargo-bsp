use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Feature(pub String);

impl Feature {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for Feature {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for Feature {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
}
