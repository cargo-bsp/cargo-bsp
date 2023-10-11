//! ParamsTarget trait implementation for the Compile/Run/Test/RustWorkspaceParams.
//! Necessary for getting the list of build targets from the compile/run/test/check requests.

use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::bsp::BuildTargetIdentifier;
use bsp_types::bsp::{CompileParams, RunParams, TestParams};
use bsp_types::rust::RustWorkspaceParams;

pub(crate) trait ParamsTarget {
    fn get_targets(&self, workspace: &ProjectWorkspace) -> Vec<BuildTargetIdentifier>;
}

impl ParamsTarget for CompileParams {
    fn get_targets(&self, _: &ProjectWorkspace) -> Vec<BuildTargetIdentifier> {
        self.targets.clone()
    }
}

impl ParamsTarget for RunParams {
    fn get_targets(&self, _: &ProjectWorkspace) -> Vec<BuildTargetIdentifier> {
        vec![self.target.clone()]
    }
}

impl ParamsTarget for TestParams {
    // Build targets for test request are sorted so that we know which target is currently tested.
    // Targets are primarily sorted by the tuple (package_name, kind, name).
    fn get_targets(&self, workspace: &ProjectWorkspace) -> Vec<BuildTargetIdentifier> {
        let mut targets = self.targets.clone();
        targets.sort_by(|id1, id2| {
            workspace
                .get_target_details(id2)
                .cmp(&workspace.get_target_details(id1))
        });
        targets
    }
}

impl ParamsTarget for RustWorkspaceParams {
    fn get_targets(&self, _: &ProjectWorkspace) -> Vec<BuildTargetIdentifier> {
        self.targets.clone()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::cargo_communication::utils::{test_package, test_target, test_target_id};
    use bsp_types::bsp::{BuildTargetIdentifier, TestParams};
    use std::collections::HashMap;

    const PACKAGE1: &str = "package1";
    const PACKAGE2: &str = "package2";
    const BIN_TARGET1: &str = "bin_target_1";
    const BIN_TARGET2: &str = "bin_target_2";
    const LIB_TARGET: &str = "lib_target";
    const TEST_TARGET1: &str = "test_target_1";
    const TEST_TARGET2: &str = "test_target_2";
    const BIN: &str = "bin";
    const LIB: &str = "lib";
    const TEST: &str = "test";

    fn test_workspace() -> ProjectWorkspace {
        let packages = vec![test_package(PACKAGE1), test_package(PACKAGE2)];

        let bin_target_1_id = test_target_id(BIN_TARGET1);
        let bin_target_2_id = test_target_id(BIN_TARGET2);
        let lib_target_id = test_target_id(LIB_TARGET);
        let test_target_1_id = test_target_id(TEST_TARGET1);
        let test_target_2_id = test_target_id(TEST_TARGET2);

        let target_id_to_package_name = HashMap::from([
            (bin_target_1_id.clone(), PACKAGE1.to_string()),
            (bin_target_2_id.clone(), PACKAGE2.to_string()),
            (lib_target_id.clone(), PACKAGE1.to_string()),
            (test_target_1_id.clone(), PACKAGE1.to_string()),
            (test_target_2_id.clone(), PACKAGE2.to_string()),
        ]);
        let target_id_to_target_data = HashMap::from([
            (bin_target_1_id, test_target(BIN_TARGET1, BIN)),
            (bin_target_2_id, test_target(BIN_TARGET2, BIN)),
            (lib_target_id, test_target(LIB_TARGET, LIB)),
            (test_target_1_id, test_target(TEST_TARGET1, TEST)),
            (test_target_2_id, test_target(TEST_TARGET2, TEST)),
        ]);
        ProjectWorkspace {
            packages,
            target_id_to_package_name,
            target_id_to_target_data,
            ..ProjectWorkspace::default()
        }
    }

    fn test_params(targets: Vec<BuildTargetIdentifier>) -> TestParams {
        TestParams {
            targets,
            ..TestParams::default()
        }
    }

    #[test]
    fn test_sorting_targets() {
        let unsorted_test_targets = vec![
            test_target_id(BIN_TARGET1),
            test_target_id(BIN_TARGET2),
            test_target_id(LIB_TARGET),
            test_target_id(TEST_TARGET1),
            test_target_id(TEST_TARGET2),
        ];
        let sorted_test_targets = vec![
            test_target_id(TEST_TARGET2),
            test_target_id(BIN_TARGET2),
            test_target_id(TEST_TARGET1),
            test_target_id(BIN_TARGET1),
            test_target_id(LIB_TARGET),
        ];
        let result = test_params(unsorted_test_targets).get_targets(&test_workspace());
        assert_eq!(result, sorted_test_targets);
    }
}
