use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum JavaExtensions {}

impl Request for JavaExtensions {
    type Params = JavacOptionsParams;
    type Result = JavacOptionsResult;
    const METHOD: &'static str = "buildTarget/javacOptions";
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavacOptionsParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavacOptionsResult {
    pub items: Vec<JavacOptionsItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JavacOptionsItem {
    pub target: BuildTargetIdentifier,

    /** Additional arguments to the compiler.
     * For example, -deprecation. */
    options: Vec<String>,

    /** The dependency classpath for this target, must be
     * identical to what is passed as arguments to
     * the -classpath flag in the command line interface
     * of javac. */
    classpath: Vec<Uri>,

    /** The output directory for classfiles produced by this target */
    class_directory: Uri,
}
