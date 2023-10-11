use serde::{Deserialize, Serialize};

use crate::*;

/// Structure to capture a description for an error code.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeDescription {
    /// An URI to open with more information about the diagnostic error.
    pub href: URI,
}
