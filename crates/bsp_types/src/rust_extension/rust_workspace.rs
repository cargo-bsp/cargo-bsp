use crate::requests::Request;
use crate::{BuildTargetIdentifier, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub packages: Vec<RustPackage>, // obcięcie do tego od czego zależą przesłane targety (od biedy wszystko)
    pub raw_dependencies: HashMap<String, RustRawDependency>, //suma dependencji pakietów targetów
    pub dependencies: HashMap<String, RustDependency>, //zmapowane RustRawDependency na RustDependency
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustRawDependency {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses_default_features: Option<bool>,
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustTarget {
    pub name: String,
    pub crate_root_url: String,
    pub package_root_url: String,
    pub kind: RustTargetKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctest: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_features: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
// todo check serialization
pub enum RustTargetKind {
    #[default]
    Bin,
    Test,
    Example,
    Bench,
    CustomBuild,
    Unknown,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustFeature {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deps: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustCfgOptions {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub key_value_options: HashMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub name_options: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustProcMacroArtifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Uri>, // path to compiled lib of proc macro .so on linux, .dll on windows .dylib on mac
                           // RUSTC_BOOTSTRAP=1 cargo check --message-format json --workspace --all-targets -Z unstable-options --keep-going | grep ""
                           // we don't need hash. It is calculated by IntelliJ-Rust
                           //pub hash: String, // ignore
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
    pub enabled_features: Vec<String>, //?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_options: Option<RustCfgOptions>, //Null or check where it comes from in current plugin implementaion
    pub env: HashMap<String, String>, //? to co ma plugin: https://github.com/intellij-rust/intellij-rust/blob/d99a5fcd5de6dd4bd81d18d67e0c6718e7612127/src/main/kotlin/org/rust/cargo/toolchain/impl/CargoMetadata.kt#L438 to co wysyła ZPP: https://github.com/ZPP-This-is-fine/bazel-bsp/blob/712e005abcd9d3f0a02a2d2001d486f2c728559e/server/src/main/java/org/jetbrains/bsp/bazel/server/sync/languages/rust/RustWorkspaceResolver.kt#L155
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_dir_url: Option<String>, // tutaj Null, bo nie mamy pojęcia co to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proc_macro_artifact: Option<RustProcMacroArtifact>, //?
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DepKind {
    pub kind: DepKindEnum,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
// todo check serialization
pub enum DepKindEnum {
    Unclassified,
    Stdlib,
    #[default]
    Normal,
    Dev,
    Build,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDependency {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dep_kinds: Vec<DepKind>,
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
            raw_dependencies: HashMap::from([(
                "package_id".to_string(),
                RustRawDependency::default(),
            )]),
            dependencies: HashMap::from([("source".to_string(), RustDependency::default())]),
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
              "env": {}
            }
          ],
          "rawDependencies": {
            "package_id": {
              "name": "",
              "features": []
            }
          },
          "dependencies": {
            "source": {
              "target": ""
            }
          }
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
            name: "test_name".to_string(),
            rename: Some("test_rename".to_string()),
            kind: Some("test_kind".to_string()),
            target: Some("test_target".to_string()),
            optional: Some(false),
            uses_default_features: Some(false),
            features: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(dependency, @r###"
        {
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
          "name": "",
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
            kind: RustTargetKind::default(),
            edition: Some("test_edition".to_string()),
            doctest: Some(false),
            required_features: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(target, @r###"
        {
          "name": "test_name",
          "crateRootUrl": "test_crate_url",
          "packageRootUrl": "test_root_url",
          "kind": "Bin",
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
          "kind": "Bin"
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
          "name": ""
        }
        "###);
    }

    #[test]
    fn rust_cfg_options() {
        let cfg_options = RustCfgOptions {
            key_value_options: HashMap::from([("key".to_string(), vec!["value".to_string()])]),
            name_options: vec!["name1".to_string(), "name2".to_string()],
        };

        assert_json_snapshot!(cfg_options, @r###"
        {
          "keyValueOptions": {
            "key": [
              "value"
            ]
          },
          "nameOptions": [
            "name1",
            "name2"
          ]
        }
        "###);

        assert_json_snapshot!(RustCfgOptions::default(), @r###"
        {}
        "###);
    }

    #[test]
    fn rust_proc_macro_artifact() {
        let proc_macro_artifact = RustProcMacroArtifact {
            path: Some("test_path".to_string()),
        };

        assert_json_snapshot!(proc_macro_artifact, @r###"
        {
          "path": "test_path"
        }
        "###);

        assert_json_snapshot!(RustProcMacroArtifact::default(), @r###"
        {}
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
            env: HashMap::from([("key".to_string(), "value".to_string())]),
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
              "kind": "Bin"
            }
          ],
          "allTargets": [
            {
              "name": "",
              "crateRootUrl": "",
              "packageRootUrl": "",
              "kind": "Bin"
            }
          ],
          "features": [
            {
              "name": ""
            }
          ],
          "enabledFeatures": [
            "feature1",
            "feature2"
          ],
          "cfgOptions": {},
          "env": {
            "key": "value"
          },
          "outDirUrl": "test_out_dir_url",
          "procMacroArtifact": {}
        }
        "###);

        assert_json_snapshot!(RustPackage::default(), @r###"
        {
          "id": "",
          "targets": [],
          "allTargets": [],
          "features": [],
          "enabledFeatures": [],
          "env": {}
        }
        "###);
    }

    #[test]
    fn rust_dep_kind_info() {
        let dep_kind_info = DepKind {
            kind: DepKindEnum::default(),
            target: Some("test_target".to_string()),
        };

        assert_json_snapshot!(dep_kind_info, @r###"
        {
          "kind": "Normal",
          "target": "test_target"
        }
        "###);

        assert_json_snapshot!(DepKind::default(), @r###"
        {
          "kind": "Normal"
        }
        "###);
    }

    #[test]
    fn rust_dependency() {
        let dependency = RustDependency {
            target: "test_target".to_string(),
            name: Some("test_name".to_string()),
            dep_kinds: vec![DepKind::default()],
        };

        assert_json_snapshot!(dependency, @r###"
        {
          "target": "test_target",
          "name": "test_name",
          "depKinds": [
            {
              "kind": "Normal"
            }
          ]
        }
        "###);

        assert_json_snapshot!(RustDependency::default(), @r###"
        {
          "target": ""
        }
        "###);
    }
}
