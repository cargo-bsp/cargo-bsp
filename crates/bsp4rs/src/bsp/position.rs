use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: i32,
    /// Character offset on a line in a document (zero-based)
    ///
    /// If the character value is greater than the line length it defaults back
    /// to the line length.
    pub character: i32,
}
