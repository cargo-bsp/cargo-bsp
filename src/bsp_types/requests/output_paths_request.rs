use crate::bsp_types::{BuildTargetIdentifier, MethodName, Uri};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathsParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

impl MethodName for OutputPathsParams {
    fn get_method_name() -> &'static str {
        "buildTarget/outputPaths"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathsResult {
    pub items: Vec<OutputPathsItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathsItem {
    /** A build target to which output paths item belongs. */
    pub target: BuildTargetIdentifier,
    /** Output paths. */
    pub output_paths: Vec<OutputPathItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathItem {
    /** Either a file or a directory. A directory entry must end with a forward
     * slash "/" and a directory entry implies that every nested path within the
     * directory belongs to this output item. */
    pub uri: Uri,

    /** Type of file of the output item, such as whether it is file or directory. */
    pub kind: OutputPathItemKind,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum OutputPathItemKind {
    /** The output path item references a normal file. */
    #[default]
    File = 1,
    /** The output path item references a directory. */
    Directory = 2,
}
