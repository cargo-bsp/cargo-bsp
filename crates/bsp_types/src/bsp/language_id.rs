use serde::{Deserialize, Serialize};

/// Language IDs are defined here
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocumentItem
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageId(pub String);

impl LanguageId {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for LanguageId {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for LanguageId {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
}
