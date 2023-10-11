use serde::{Deserialize, Serialize};

use crate::*;

/// Build target contains metadata about an artifact (for example library, test, or binary artifact). Using vocabulary of other build tools:
///
/// * sbt: a build target is a combined project + config. Example:
/// * a regular JVM project with main and test configurations will have 2 build targets, one for main and one for test.
/// * a single configuration in a single project that contains both Java and Scala sources maps to one BuildTarget.
/// * a project with crossScalaVersions 2.11 and 2.12 containing main and test configuration in each will have 4 build targets.
/// * a Scala 2.11 and 2.12 cross-built project for Scala.js and the JVM with main and test configurations will have 8 build targets.
/// * Pants: a pants target corresponds one-to-one with a BuildTarget
/// * Bazel: a bazel target corresponds one-to-one with a BuildTarget
///
/// The general idea is that the BuildTarget data structure should contain only information that is fast or cheap to compute.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTarget {
    /// The target's unique identifier
    pub id: BuildTargetIdentifier,
    /// A human readable name for this target.
    /// May be presented in the user interface.
    /// Should be unique if possible.
    /// The id.uri is used if None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// The directory where this target belongs to. Multiple build targets are allowed to map
    /// to the same base directory, and a build target is not required to have a base directory.
    /// A base directory does not determine the sources of a target, see buildTarget/sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_directory: Option<URI>,
    /// Free-form string tags to categorize or label this build target.
    /// For example, can be used by the client to:
    /// - customize how the target should be translated into the client's project model.
    /// - group together different but related targets in the user interface.
    /// - display icons or colors in the user interface.
    /// Pre-defined tags are listed in `BuildTargetTag` but clients and servers
    /// are free to define new tags for custom purposes.
    pub tags: Vec<BuildTargetTag>,
    /// The set of languages that this target contains.
    /// The ID string for each language is defined in the LSP.
    pub language_ids: Vec<LanguageId>,
    /// The direct upstream build target dependencies of this build target
    pub dependencies: Vec<BuildTargetIdentifier>,
    /// The capabilities of this build target.
    pub capabilities: BuildTargetCapabilities,
    /// Language-specific metadata about this target.
    /// See ScalaBuildTarget as an example.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<BuildTargetData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn build_target() {
        let test_data = BuildTarget {
            id: BuildTargetIdentifier::default(),
            display_name: Some("test_displayName".to_string()),
            base_directory: Some("test_baseDirectory".into()),
            tags: vec![BuildTargetTag::TEST],
            capabilities: BuildTargetCapabilities::default(),
            language_ids: vec!["test_languageId".into()],
            dependencies: vec![BuildTargetIdentifier::default()],
            data: Some(BuildTargetData::cargo(CargoBuildTarget::default())),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "id": {
            "uri": ""
          },
          "displayName": "test_displayName",
          "baseDirectory": "test_baseDirectory",
          "tags": [
            "test"
          ],
          "languageIds": [
            "test_languageId"
          ],
          "dependencies": [
            {
              "uri": ""
            }
          ],
          "capabilities": {},
          "dataKind": "cargo",
          "data": {
            "edition": "",
            "requiredFeatures": []
          }
        }
        "#
        );
        assert_json_snapshot!(BuildTarget::default(),
            @r#"
        {
          "id": {
            "uri": ""
          },
          "tags": [],
          "languageIds": [],
          "dependencies": [],
          "capabilities": {}
        }
        "#
        );
    }
}
