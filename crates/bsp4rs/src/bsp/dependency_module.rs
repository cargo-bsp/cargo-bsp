use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModule {
    /// Module name
    pub name: String,
    /// Module version
    pub version: String,
    /// Language-specific metadata about this module.
    /// See MavenDependencyModule as an example.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<DependencyModuleData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

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
