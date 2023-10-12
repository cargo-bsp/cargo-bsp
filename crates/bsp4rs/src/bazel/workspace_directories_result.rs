use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDirectoriesResult {
    pub included_directories: Vec<DirectoryItem>,
    pub excluded_directories: Vec<DirectoryItem>,
}
