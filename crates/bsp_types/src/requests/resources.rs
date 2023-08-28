use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum Resources {}

impl Request for Resources {
    type Params = ResourcesParams;
    type Result = ResourcesResult;
    const METHOD: &'static str = "buildTarget/resources";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourcesResult {
    pub items: Vec<ResourcesItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct ResourcesItem {
    pub target: BuildTargetIdentifier,
    /** List of resource files. */
    pub resources: Vec<Uri>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn resources_method() {
        assert_eq!(Resources::METHOD, "buildTarget/resources");
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
            resources: vec![Uri::default()],
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
