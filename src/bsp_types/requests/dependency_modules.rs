use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::bsp_types::requests::Request;
use crate::bsp_types::BuildTargetIdentifier;

#[derive(Debug)]
pub enum DependencyModules {}

impl Request for DependencyModules {
    type Params = DependencyModulesParams;
    type Result = DependencyModulesResult;
    const METHOD: &'static str = "buildTarget/dependencyModules";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DependencyModulesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DependencyModulesResult {
    pub items: Vec<DependencyModulesItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct DependencyModulesItem {
    pub target: BuildTargetIdentifier,
    pub modules: Vec<DependencyModule>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModule {
    /** Module name */
    pub name: String,

    /** Module version */
    pub version: String,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata about this module.
    See MavenDependencyModule as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::bsp_types::tests::test_deserialization;

    use super::*;

    #[test]
    fn dependency_modules_method() {
        assert_eq!(DependencyModules::METHOD, "buildTarget/dependencyModules");
    }

    #[test]
    fn dependency_modules_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &DependencyModulesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &DependencyModulesParams::default());
    }

    #[test]
    fn dependency_modules_result() {
        let test_data = DependencyModulesResult {
            items: vec![DependencyModulesItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "items": [
            {
              "target": {
                "uri": ""
              },
              "modules": []
            }
          ]
        }
        "###
        );
        assert_json_snapshot!(DependencyModulesResult::default(),
            @r###"
        {
          "items": []
        }
        "###
        );
    }

    #[test]
    fn dependency_modules_item() {
        let test_data = DependencyModulesItem {
            target: BuildTargetIdentifier::default(),
            modules: vec![DependencyModule::default()],
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "target": {
            "uri": ""
          },
          "modules": [
            {
              "name": "",
              "version": ""
            }
          ]
        }
        "###
        );
        assert_json_snapshot!(DependencyModulesItem::default(),
            @r###"
        {
          "target": {
            "uri": ""
          },
          "modules": []
        }
        "###
        );
    }

    #[test]
    fn dependency_module() {
        let test_data = DependencyModule {
            name: "test_name".to_string(),
            version: "test_version".to_string(),
            data_kind: Some("test_dataKind".to_string()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "name": "test_name",
          "version": "test_version",
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "###
        );
        assert_json_snapshot!(DependencyModule::default(),
            @r###"
        {
          "name": "",
          "version": ""
        }
        "###
        );
    }
}
