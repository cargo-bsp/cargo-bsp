use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, OtherData};

#[derive(Debug)]
pub enum BuildTargetDependencyModules {}

impl Request for BuildTargetDependencyModules {
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

    /** Language-specific metadata about this module.
    See MavenDependencyModule as an example. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<DependencyModuleData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedDependencyModuleData {}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DependencyModuleData {
    Named(NamedDependencyModuleData),
    Other(OtherData),
}

impl DependencyModuleData {}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn dependency_modules_method() {
        assert_eq!(
            BuildTargetDependencyModules::METHOD,
            "buildTarget/dependencyModules"
        );
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
            @r#"
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
        "#
        );
        assert_json_snapshot!(DependencyModulesResult::default(),
            @r#"
        {
          "items": []
        }
        "#
        );
    }

    #[test]
    fn dependency_modules_item() {
        let test_data = DependencyModulesItem {
            target: BuildTargetIdentifier::default(),
            modules: vec![DependencyModule::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
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
        "#
        );
        assert_json_snapshot!(DependencyModulesItem::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "modules": []
        }
        "#
        );
    }

    #[test]
    fn dependency_module() {
        let test_data = DependencyModule {
            name: "test_name".to_string(),
            version: "test_version".to_string(),
            data: Some(DependencyModuleData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "name": "test_name",
          "version": "test_version",
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(DependencyModule::default(),
            @r#"
        {
          "name": "",
          "version": ""
        }
        "#
        );
    }
}
