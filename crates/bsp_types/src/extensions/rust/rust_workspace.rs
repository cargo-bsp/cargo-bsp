use crate::requests::Request;
use crate::{BuildTargetIdentifier, Uri};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

#[derive(Debug)]
pub enum RustWorkspace {}

impl Request for RustWorkspace {
    type Params = RustWorkspaceParams;
    type Result = RustWorkspaceResult;
    const METHOD: &'static str = "buildTarget/rustWorkspace";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceResult {
    pub packages: Vec<RustPackage>,
    pub raw_dependencies: HashMap<String, RustRawDependency>, //packaceId -> RustDependecies //suma dependencji pakietów targetów (1)zdobądź wszystkie pakiety targetów (2) dostań ich zależności
    pub dependencies: HashMap<String, RustDependency>, //zmapowane RustRawDependency na RustDependency (1)weź każdą zależność i znajdź jej cargo_metadata::Package.source
    pub resolved_targets: Vec<BuildTargetIdentifier>,
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
    // TODO Removed Option type, check
    pub optional: bool,
    // TODO Removed Option type, check
    pub uses_default_features: bool,
    pub features: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RustTarget {
    pub name: String,
    pub crate_root_url: String,
    pub package_root_url: String,
    pub kind: RustTargetKind,
    // TODO Removed Option type, check
    pub edition: RustEdition,
    // TODO Removed Option type, check
    pub doctest: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_features: Vec<String>,
}

#[derive(Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum RustTargetKind {
    #[default]
    Lib = 1,
    Bin = 2,
    Test = 3,
    Example = 4,
    Bench = 5,
    CustomBuild = 6,
    Unknown = 7,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RustPackageOrigin {
    Stdlib,
    #[default]
    Workspace,
    Dependency,
    StdlibDependency,
    #[serde(skip_serializing)]
    // Not part of the protocol, used internally to mark that origin is not set
    Unset,
}

#[derive(Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u16)]
pub enum RustEdition {
    Edition2015 = 2015,
    #[default]
    Edition2018 = 2018,
    Edition2021 = 2021,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustFeature {
    pub name: String,
    pub dependencies: Vec<String>,
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
    pub version: String,
    pub origin: RustPackageOrigin,
    pub edition: RustEdition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub targets: Vec<RustTarget>,
    pub all_targets: Vec<RustTarget>,
    pub features: Vec<RustFeature>,
    pub enabled_features: Vec<String>, // todo resolve from Cargo metadata -> resolved -> nodes (grouped by packageId) -> features.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_options: Option<RustCfgOptions>, //Null or check where it comes from in current plugin implementaion
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>, //? to co ma plugin: https://github.com/intellij-rust/intellij-rust/blob/d99a5fcd5de6dd4bd81d18d67e0c6718e7612127/src/main/kotlin/org/rust/cargo/toolchain/impl/CargoMetadata.kt#L438 to co wysyła ZPP: https://github.com/ZPP-This-is-fine/bazel-bsp/blob/712e005abcd9d3f0a02a2d2001d486f2c728559e/server/src/main/java/org/jetbrains/bsp/bazel/server/sync/languages/rust/RustWorkspaceResolver.kt#L155
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_dir_url: Option<String>, // tutaj Null, bo nie mamy pojęcia co to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proc_macro_artifact: Option<RustProcMacroArtifact>, //?
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDepKindInfo {
    pub kind: RustDepKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum RustDepKind {
    Unclassified = 1,
    Stdlib = 2,
    #[default]
    Normal = 3,
    Dev = 4,
    Build = 5,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDependency {
    pub target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dep_kinds: Vec<RustDepKindInfo>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::test_deserialization;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_workspace_method() {
        assert_eq!(RustWorkspace::METHOD, "buildTarget/rustWorkspace");
    }

    #[test]
    fn rust_workspace_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &RustWorkspaceParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &RustWorkspaceParams::default());
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
            resolved_targets: vec![BuildTargetIdentifier::default()],
        };

        assert_json_snapshot!(result, @r###"
        {
          "packages": [
            {
              "id": "",
              "version": "",
              "origin": "workspace",
              "edition": 2018,
              "targets": [],
              "allTargets": [],
              "features": [],
              "enabledFeatures": []
            }
          ],
          "rawDependencies": {
            "package_id": {
              "name": "",
              "optional": false,
              "usesDefaultFeatures": false,
              "features": []
            }
          },
          "dependencies": {
            "source": {
              "target": ""
            }
          },
          "resolvedTargets": [
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
    fn rust_raw_dependency() {
        let dependency = RustRawDependency {
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
            kind: RustTargetKind::default(),
            edition: RustEdition::default(),
            doctest: false,
            required_features: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(target, @r###"
        {
          "name": "test_name",
          "crateRootUrl": "test_crate_url",
          "packageRootUrl": "test_root_url",
          "kind": 1,
          "edition": 2018,
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
          "kind": 1,
          "edition": 2018,
          "doctest": false
        }
        "###);
    }

    #[test]
    fn rust_feature() {
        let feature = RustFeature {
            name: "test_name".to_string(),
            dependencies: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(feature, @r###"
        {
          "name": "test_name",
          "dependencies": [
            "test_feature"
          ]
        }
        "###);

        assert_json_snapshot!(RustFeature::default(), @r###"
        {
          "name": "",
          "dependencies": []
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
            version: "test_version".to_string(),
            origin: RustPackageOrigin::default(),
            edition: RustEdition::default(),
            source: Some("test_source".to_string()),
            targets: vec![RustTarget::default()],
            all_targets: vec![RustTarget::default()],
            features: vec![RustFeature::default()],
            enabled_features: vec!["test_feature".to_string()],
            cfg_options: Some(RustCfgOptions::default()),
            env: HashMap::from([("key".to_string(), "value".to_string())]),
            out_dir_url: Some("test_out_dir_url".to_string()),
            proc_macro_artifact: Some(RustProcMacroArtifact::default()),
        };

        assert_json_snapshot!(package, @r###"
        {
          "id": "test_id",
          "version": "test_version",
          "origin": "workspace",
          "edition": 2018,
          "source": "test_source",
          "targets": [
            {
              "name": "",
              "crateRootUrl": "",
              "packageRootUrl": "",
              "kind": 1,
              "edition": 2018,
              "doctest": false
            }
          ],
          "allTargets": [
            {
              "name": "",
              "crateRootUrl": "",
              "packageRootUrl": "",
              "kind": 1,
              "edition": 2018,
              "doctest": false
            }
          ],
          "features": [
            {
              "name": "",
              "dependencies": []
            }
          ],
          "enabledFeatures": [
            "test_feature"
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
          "version": "",
          "origin": "workspace",
          "edition": 2018,
          "targets": [],
          "allTargets": [],
          "features": [],
          "enabledFeatures": []
        }
        "###);
    }

    #[test]
    fn rust_dep_kind_info() {
        let dep_kind_info = RustDepKindInfo {
            kind: RustDepKind::default(),
            target: Some("test_target".to_string()),
        };

        assert_json_snapshot!(dep_kind_info, @r###"
        {
          "kind": 3,
          "target": "test_target"
        }
        "###);

        assert_json_snapshot!(RustDepKindInfo::default(), @r###"
        {
          "kind": 3
        }
        "###);
    }

    #[test]
    fn rust_dependency() {
        let dependency = RustDependency {
            target: "test_target".to_string(),
            name: Some("test_name".to_string()),
            dep_kinds: vec![RustDepKindInfo::default()],
        };

        assert_json_snapshot!(dependency, @r###"
        {
          "target": "test_target",
          "name": "test_name",
          "depKinds": [
            {
              "kind": 3
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

    #[test]
    fn rust_target_kind() {
        assert_json_snapshot!(RustTargetKind::Lib, @"1");
        assert_json_snapshot!(RustTargetKind::Bin, @"2");
        assert_json_snapshot!(RustTargetKind::Test, @"3");
        assert_json_snapshot!(RustTargetKind::Example, @"4");
        assert_json_snapshot!(RustTargetKind::Bench, @"5");
        assert_json_snapshot!(RustTargetKind::CustomBuild, @"6");
        assert_json_snapshot!(RustTargetKind::Unknown, @"7");
    }

    #[test]
    fn rust_dep_kind() {
        assert_json_snapshot!(RustDepKind::Unclassified, @"1");
        assert_json_snapshot!(RustDepKind::Stdlib, @"2");
        assert_json_snapshot!(RustDepKind::Normal, @"3");
        assert_json_snapshot!(RustDepKind::Dev, @"4");
        assert_json_snapshot!(RustDepKind::Build, @"5");
    }

    #[test]
    fn rust_edition() {
        assert_json_snapshot!(RustEdition::Edition2015, @"2015");
        assert_json_snapshot!(RustEdition::Edition2018, @"2018");
        assert_json_snapshot!(RustEdition::Edition2021, @"2021");
    }

    #[test]
    fn rust_package_origin() {
        assert_json_snapshot!(RustPackageOrigin::Stdlib, @r###""stdlib""###);
        assert_json_snapshot!(RustPackageOrigin::Workspace, @r###""workspace""###);
        assert_json_snapshot!(RustPackageOrigin::Dependency, @r###""dependency""###);
        assert_json_snapshot!(RustPackageOrigin::StdlibDependency, @r###""stdlib-dependency""###);
    }
}
