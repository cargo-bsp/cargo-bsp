use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::BuildTarget;

#[derive(Debug)]
pub enum WorkspaceBuildTargets {}

impl Request for WorkspaceBuildTargets {
    type Params = (); // TODO change to WorkspaceBuildTargetsParams if client supports
    type Result = WorkspaceBuildTargetsResult;
    const METHOD: &'static str = "workspace/buildTargets";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkspaceBuildTargetsParams {}

/** The result of the workspace/buildTargets request */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct WorkspaceBuildTargetsResult {
    /** The build targets in this workspace that
     * contain sources with the given language ids. */
    pub targets: Vec<BuildTarget>,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn workspace_build_targets_method() {
        assert_eq!(WorkspaceBuildTargets::METHOD, "workspace/buildTargets");
    }

    #[test]
    fn workspace_build_targets_params() {
        test_deserialization(r#"{}"#, &WorkspaceBuildTargetsParams {});
    }

    #[test]
    fn inverse_sources_result() {
        test_serialization(
            &WorkspaceBuildTargetsResult {
                targets: vec![BuildTarget::default()],
            },
            r#"{"targets":[{"id":{"uri":""},"tags":[],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":[],"dependencies":[]}]}"#,
        );
        test_serialization(
            &WorkspaceBuildTargetsResult { targets: vec![] },
            r#"{"targets":[]}"#,
        );
    }
}
