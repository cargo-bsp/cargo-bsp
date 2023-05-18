use crate::requests::Request;
use crate::BuildTargetIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RustWorkspace {}

impl Request for RustWorkspace {
    type Params = RustWorkspaceParams;
    type Result = RustWorkspaceResult;
    const METHOD: &'static str = "buildTarget/rustWorkspace";
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceResult {
    pub packages: Vec<RustPackage>,
    pub raw_dependencies: Vec<RustRawDependency>,
    pub dependencies: Vec<RustDependency>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustRawDependency {
    pub package_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub optional: bool,
    pub uses_default_features: bool,
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustTarget {
    pub name: String,
    pub crate_root_url: String,
    pub package_root_url: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    pub doctest: bool,
    pub required_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustFeature {
    pub name: String,
    pub deps: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustEnvData {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustKeyValueMapper {
    pub key: String,
    pub value: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustCfgOptions {
    pub key_value_options: Vec<RustKeyValueMapper>,
    pub name_options: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustProcMacroArtifact {
    pub path: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustPackage {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub targets: Vec<RustTarget>,
    pub all_targets: Vec<RustTarget>,
    pub features: Vec<RustFeature>,
    pub enabled_features: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_options: Option<RustCfgOptions>,
    pub env: Vec<RustEnvData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_dir_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proc_macro_artifact: Option<RustProcMacroArtifact>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDepKindInfo {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDependency {
    pub source: String,
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub dep_kinds: Vec<RustDepKindInfo>,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_workspace_method() {
        assert_eq!(RustWorkspace::METHOD, "buildTarget/rustWorkspace");
    }

    #[test]
    fn rust_workspace_params() {
        let params = RustWorkspaceParams {
            targets: vec![BuildTargetIdentifier::default()],
        };

        assert_json_snapshot!(params, @r###"
        {
          "targets": [
            {
              "uri": ""
            }
          ]
        }
        "###);

        assert_json_snapshot!(RustWorkspaceParams::default(), @r###"
        {
          "targets": []
        }
        "###);
    }

    #[test]
    fn rust_workspace_result() {
        let result = RustWorkspaceResult {
            packages: vec![RustPackage::default()],
            raw_dependencies: vec![RustRawDependency::default()],
            dependencies: vec![RustDependency::default()],
        };

        assert_json_snapshot!(result, @r###"
        {
          "packages": [
            {
              "id": "",
              "targets": [],
              "allTargets": [],
              "features": [],
              "enabledFeatures": [],
              "env": []
            }
          ],
          "rawDependencies": [
            {
              "packageId": "",
              "name": "",
              "optional": false,
              "usesDefaultFeatures": false,
              "features": []
            }
          ],
          "dependencies": [
            {
              "source": "",
              "target": "",
              "depKinds": []
            }
          ]
        }
        "###);

        assert_json_snapshot!(RustWorkspaceParams::default(), @r###"
        {
          "targets": []
        }
        "###);
    }

    #[test]
    fn rust_raw_dependency() {
        let dependency = RustRawDependency {
            package_id: "test_id".to_string(),
            name: "test_name".to_string(),
            rename: Some("test_rename".to_string()),
            kind: Some("test_kind".to_string()),
            target: Some("test_target".to_string()),
            optional: false,
            uses_default_features: false,
            features: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(dependency, @r###"
        {
          "packageId": "test_id",
          "name": "test_name",
          "rename": "test_rename",
          "kind": "test_kind",
          "target": "test_target",
          "optional": false,
          "usesDefaultFeatures": false,
          "features": [
            "test_feature"
          ]
        }
        "###);

        assert_json_snapshot!(RustRawDependency::default(), @r###"
        {
          "packageId": "",
          "name": "",
          "optional": false,
          "usesDefaultFeatures": false,
          "features": []
        }
        "###);
    }

    #[test]
    fn rust_target() {
        let target = RustTarget {
            name: "test_name".to_string(),
            crate_root_url: "test_crate_url".to_string(),
            package_root_url: "test_root_url".to_string(),
            kind: "test_kind".to_string(),
            edition: Some("test_edition".to_string()),
            doctest: false,
            required_features: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(target, @r###"
        {
          "name": "test_name",
          "crateRootUrl": "test_crate_url",
          "packageRootUrl": "test_root_url",
          "kind": "test_kind",
          "edition": "test_edition",
          "doctest": false,
          "requiredFeatures": [
            "test_feature"
          ]
        }
        "###);

        assert_json_snapshot!(RustTarget::default(), @r###"
        {
          "name": "",
          "crateRootUrl": "",
          "packageRootUrl": "",
          "kind": "",
          "doctest": false,
          "requiredFeatures": []
        }
        "###);
    }

    #[test]
    fn rust_feature() {
        let feature = RustFeature {
            name: "test_name".to_string(),
            deps: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(feature, @r###"
        {
          "name": "test_name",
          "deps": [
            "test_feature"
          ]
        }
        "###);

        assert_json_snapshot!(RustFeature::default(), @r###"
        {
          "name": "",
          "deps": []
        }
        "###);
    }

    #[test]
    fn rust_env_data() {
        let env_data = RustEnvData {
            name: "test_name".to_string(),
            value: "test_value".to_string(),
        };

        assert_json_snapshot!(env_data, @r###"
        {
          "name": "test_name",
          "value": "test_value"
        }
        "###);

        assert_json_snapshot!(RustEnvData::default(), @r###"
        {
          "name": "",
          "value": ""
        }
        "###);
    }

    #[test]
    fn rust_key_value_mapper() {
        let key_value_mapper = RustKeyValueMapper {
            key: "test_key".to_string(),
            value: vec!["test_value".to_string()],
        };

        assert_json_snapshot!(key_value_mapper, @r###"
        {
          "key": "test_key",
          "value": [
            "test_value"
          ]
        }
        "###);

        assert_json_snapshot!(RustKeyValueMapper::default(), @r###"
        {
          "key": "",
          "value": []
        }
        "###);
    }

    #[test]
    fn rust_cfg_options() {
        let cfg_options = RustCfgOptions {
            key_value_options: vec![
                RustKeyValueMapper {
                    key: "key1".to_string(),
                    value: vec!["value1".to_string()],
                },
                RustKeyValueMapper {
                    key: "key2".to_string(),
                    value: vec!["value2".to_string()],
                },
            ],
            name_options: vec!["name1".to_string(), "name2".to_string()],
        };

        assert_json_snapshot!(cfg_options, @r###"
        {
          "keyValueOptions": [
            {
              "key": "key1",
              "value": [
                "value1"
              ]
            },
            {
              "key": "key2",
              "value": [
                "value2"
              ]
            }
          ],
          "nameOptions": [
            "name1",
            "name2"
          ]
        }
        "###);

        assert_json_snapshot!(RustCfgOptions::default(), @r###"
        {
          "keyValueOptions": [],
          "nameOptions": []
        }
        "###);
    }

    #[test]
    fn rust_proc_macro_artifact() {
        let proc_macro_artifact = RustProcMacroArtifact {
            path: "test_path".to_string(),
            hash: "test_hash".to_string(),
        };

        assert_json_snapshot!(proc_macro_artifact, @r###"
        {
          "path": "test_path",
          "hash": "test_hash"
        }
        "###);

        assert_json_snapshot!(RustProcMacroArtifact::default(), @r###"
        {
          "path": "",
          "hash": ""
        }
        "###);
    }

    #[test]
    fn rust_package() {
        let package = RustPackage {
            id: "test_id".to_string(),
            version: Some("test_version".to_string()),
            origin: Some("test_origin".to_string()),
            edition: Some("test_edition".to_string()),
            source: Some("test_source".to_string()),
            targets: vec![RustTarget::default()],
            all_targets: vec![RustTarget::default()],
            features: vec![RustFeature::default()],
            enabled_features: vec!["feature1".to_string(), "feature2".to_string()],
            cfg_options: Some(RustCfgOptions::default()),
            env: vec![RustEnvData::default()],
            out_dir_url: Some("test_out_dir_url".to_string()),
            proc_macro_artifact: Some(RustProcMacroArtifact::default()),
        };

        assert_json_snapshot!(package, @r###"
        {
          "id": "test_id",
          "version": "test_version",
          "origin": "test_origin",
          "edition": "test_edition",
          "source": "test_source",
          "targets": [
            {
              "name": "",
              "crateRootUrl": "",
              "packageRootUrl": "",
              "kind": "",
              "doctest": false,
              "requiredFeatures": []
            }
          ],
          "allTargets": [
            {
              "name": "",
              "crateRootUrl": "",
              "packageRootUrl": "",
              "kind": "",
              "doctest": false,
              "requiredFeatures": []
            }
          ],
          "features": [
            {
              "name": "",
              "deps": []
            }
          ],
          "enabledFeatures": [
            "feature1",
            "feature2"
          ],
          "cfgOptions": {
            "keyValueOptions": [],
            "nameOptions": []
          },
          "env": [
            {
              "name": "",
              "value": ""
            }
          ],
          "outDirUrl": "test_out_dir_url",
          "procMacroArtifact": {
            "path": "",
            "hash": ""
          }
        }
        "###);

        assert_json_snapshot!(RustPackage::default(), @r###"
        {
          "id": "",
          "targets": [],
          "allTargets": [],
          "features": [],
          "enabledFeatures": [],
          "env": []
        }
        "###);
    }

    #[test]
    fn rust_dep_kind_info() {
        let dep_kind_info = RustDepKindInfo {
            kind: "test_kind".to_string(),
            target: Some("test_target".to_string()),
        };

        assert_json_snapshot!(dep_kind_info, @r###"
        {
          "kind": "test_kind",
          "target": "test_target"
        }
        "###);

        assert_json_snapshot!(RustDepKindInfo::default(), @r###"
        {
          "kind": ""
        }
        "###);
    }

    #[test]
    fn rust_dependency() {
        let dependency = RustDependency {
            source: "test_source".to_string(),
            target: "test_target".to_string(),
            name: Some("test_name".to_string()),
            dep_kinds: vec![RustDepKindInfo::default()],
        };

        assert_json_snapshot!(dependency, @r###"
        {
          "source": "test_source",
          "target": "test_target",
          "name": "test_name",
          "depKinds": [
            {
              "kind": ""
            }
          ]
        }
        "###);

        assert_json_snapshot!(RustDependency::default(), @r###"
        {
          "source": "",
          "target": "",
          "depKinds": []
        }
        "###);
    }
}
