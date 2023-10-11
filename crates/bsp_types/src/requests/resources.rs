use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, URI};

/// The build target resources request is sent from the client to the server to
/// query for the list of resources of a given list of build targets.
///
/// A resource is a data dependency required to be present in the runtime classpath
/// when a build target is run or executed. The server communicates during the
/// initialize handshake whether this method is supported or not.
///
/// This request can be used by a client to highlight the resources in a project
/// view, for example.
#[derive(Debug)]
pub enum BuildTargetResources {}

impl Request for BuildTargetResources {
    type Params = ResourcesParams;
    type Result = ResourcesResult;
    const METHOD: &'static str = "buildTarget/resources";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesResult {
    pub items: Vec<ResourcesItem>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesItem {
    pub target: BuildTargetIdentifier,
    /// List of resource files.
    pub resources: Vec<URI>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn resources_method() {
        assert_eq!(BuildTargetResources::METHOD, "buildTarget/resources");
    }

    #[test]
    fn resources_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &ResourcesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &ResourcesParams::default());
    }

    #[test]
    fn resources_result() {
        let test_data = ResourcesResult {
            items: vec![ResourcesItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "items": [
            {
              "target": {
                "uri": ""
              },
              "resources": []
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(ResourcesResult::default(),
            @r#"
        {
          "items": []
        }
        "#
        );
    }

    #[test]
    fn resources_item() {
        let test_data = ResourcesItem {
            target: BuildTargetIdentifier::default(),
            resources: vec![URI::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "resources": [
            ""
          ]
        }
        "#
        );
        assert_json_snapshot!(ResourcesItem::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "resources": []
        }
        "#
        );
    }
}
