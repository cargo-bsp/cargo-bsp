use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::URI;

#[derive(Debug)]
pub enum WorkspaceDirectories {}

impl Request for WorkspaceDirectories {
    type Params = ();
    type Result = WorkspaceDirectoriesResult;
    const METHOD: &'static str = "workspace/directories";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDirectoriesResult {
    pub included_directories: Vec<DirectoryItem>,
    pub excluded_directories: Vec<DirectoryItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryItem {
    pub uri: URI,
}
