use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceResult {
    /// Packages of given targets.
    pub packages: Vec<RustPackage>,
    /// Dependencies in `cargo metadata` as listed in the package `Cargo.toml`,
    /// without package resolution or any additional data.
    pub raw_dependencies: RustRawDependencies,
    /// Resolved dependencies of the build. Handles renamed dependencies.
    /// Correspond to dependencies from resolved dependency graph from `cargo metadata` that shows
    /// the actual dependencies that are being used in the build.
    pub dependencies: RustDependencies,
    /// A sequence of build targets taken into consideration during build process.
    pub resolved_targets: Vec<BuildTargetIdentifier>,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;
    use std::collections::BTreeMap;

    #[test]
    fn rust_workspace_result() {
        let result = RustWorkspaceResult {
            packages: vec![RustPackage::default()],
            raw_dependencies: RustRawDependencies::new(BTreeMap::from([(
                "package_id".to_string(),
                vec![RustRawDependency::default()],
            )])),
            dependencies: RustDependencies::new(BTreeMap::from([(
                "package_id".to_string(),
                vec![RustDependency::default()],
            )])),
            resolved_targets: vec![BuildTargetIdentifier::default()],
        };

        assert_json_snapshot!(result, @r#"
        {
          "packages": [
            {
              "id": "",
              "rootUrl": "",
              "name": "",
              "version": "",
              "origin": "",
              "edition": "",
              "resolvedTargets": [],
              "allTargets": [],
              "features": {},
              "enabledFeatures": []
            }
          ],
          "rawDependencies": {
            "package_id": [
              {
                "name": "",
                "optional": false,
                "usesDefaultFeatures": false,
                "features": []
              }
            ]
          },
          "dependencies": {
            "package_id": [
              {
                "pkg": ""
              }
            ]
          },
          "resolvedTargets": [
            {
              "uri": ""
            }
          ]
        }
        "#);

        assert_json_snapshot!(RustWorkspaceParams::default(), @r#"
        {
          "targets": []
        }
        "#);
    }
}
