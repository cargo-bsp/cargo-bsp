use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, URI};

#[derive(Debug)]
pub enum BuildTargetDependencySources {}

/// The build target dependency sources request is sent from the client to the
/// server to query for the sources of build target dependencies that are external
/// to the workspace. The dependency sources response must not include source files
/// that belong to a build target within the workspace, see `buildTarget/sources`.
///
/// The server communicates during the initialize handshake whether this method is
/// supported or not. This method can for example be used by a language server on
/// `textDocument/definition` to "Go to definition" from project sources to
/// dependency sources.
impl Request for BuildTargetDependencySources {
    type Params = DependencySourcesParams;
    type Result = DependencySourcesResult;
    const METHOD: &'static str = "buildTarget/dependencySources";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesResult {
    pub items: Vec<DependencySourcesItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesItem {
    pub target: BuildTargetIdentifier,
    /// List of resources containing source files of the
    /// target's dependencies.
    /// Can be source files, jar files, zip files, or directories.
    pub sources: Vec<URI>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn dependency_sources_method() {
        assert_eq!(
            BuildTargetDependencySources::METHOD,
            "buildTarget/dependencySources"
        );
    }

    #[test]
    fn dependency_sources_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &DependencySourcesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &DependencySourcesParams::default());
    }

    #[test]
    fn dependency_sources_result() {
        let test_data = DependencySourcesResult {
            items: vec![DependencySourcesItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "items": [
            {
              "target": {
                "uri": ""
              },
              "sources": []
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(DependencySourcesResult::default(),
            @r#"
        {
          "items": []
        }
        "#
        );
    }

    #[test]
    fn dependency_sources_item() {
        let test_data = DependencySourcesItem {
            target: BuildTargetIdentifier::default(),
            sources: vec![URI::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "sources": [
            ""
          ]
        }
        "#
        );
        assert_json_snapshot!(DependencySourcesItem::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "sources": []
        }
        "#
        );
    }
}
