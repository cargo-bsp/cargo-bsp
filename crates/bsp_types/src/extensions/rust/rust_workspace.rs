use crate::requests::Request;
use crate::{BuildTargetIdentifier, Edition, Uri};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::{BTreeSet, HashMap};

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
    /** A sequence of build targets for workspace resolution. */
    pub targets: Vec<BuildTargetIdentifier>,
}

pub type PackageIdToRustRawDependency = HashMap<String, Vec<RustRawDependency>>;
pub type PackageIdToRustDependency = HashMap<String, Vec<RustDependency>>;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceResult {
    /** Packages of given targets. */
    pub packages: Vec<RustPackage>,
    /** Dependencies as listed in the package `Cargo.toml`,
    without package resolution or any additional data. */
    pub raw_dependencies: PackageIdToRustRawDependency,
    /** Resolved dependencies of the package. Handles renamed dependencies. */
    pub dependencies: PackageIdToRustDependency,
    /** A sequence of build targets taken into consideration during build process. */
    pub resolved_targets: Vec<BuildTargetIdentifier>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustRawDependency {
    /** The name of the dependency. */
    pub name: String,
    /** Name to which this dependency is renamed when declared in Cargo.toml. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    /** The dependency kind. "dev", "build", or null for a normal dependency. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /** The target platform for the dependency. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /** Indicates whether this is an optional dependency. */
    pub optional: bool,
    /** Indicates whether default features are enabled. */
    pub uses_default_features: bool,
    /** A sequence of enabled features. **/
    pub features: BTreeSet<String>,
}

/** This structure is embedded in the `data?: BuildTargetData` field, when the
`dataKind` field contains "rust". */
#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RustBuildTarget {
    /** The name of the target. */
    pub name: String,
    /** Path to the root module of the crate. */
    pub crate_root_url: Uri,
    /** A target's kind. */
    pub kind: RustTargetKind,
    /** Type of output that is produced by a crate during the build process.
    The crate type determines how the source code is compiled. */
    pub crate_types: Vec<RustCrateType>,
    /** The Rust edition of the target. */
    pub edition: Edition,
    /** Whether or not this target has doc tests enabled, and
    the target is compatible with doc testing. */
    pub doctest: bool,
    /** A sequence of required features. */
    pub required_features: BTreeSet<String>,
}

#[derive(Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum RustTargetKind {
    /** For lib targets. */
    #[default]
    Lib = 1,
    /** For binaries. */
    Bin = 2,
    /** For integration tests. */
    Test = 3,
    /** For examples. */
    Example = 4,
    /** For benchmarks. */
    Bench = 5,
    /** For build scripts. */
    CustomBuild = 6,
    /** For unknown targets. */
    Unknown = 7,
}

/** Crate types (`lib`, `rlib`, `dylib`, `cdylib`, `staticlib`) are listed for
`lib` and `example` target kinds. For other target kinds `bin` crate type is listed. */
#[derive(Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum RustCrateType {
    Bin = 1,
    #[default]
    Lib = 2,
    Rlib = 3,
    Dylib = 4,
    Cdylib = 5,
    Staticlib = 6,
    ProcMacro = 7,
    Unknown = 8,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RustPackageOrigin {
    /** The package comes from the standard library. */
    Stdlib,
    /** The package is a part of our workspace. */
    #[default]
    Workspace,
    /** External dependency of [WORKSPACE] or other [DEPENDENCY] package. */
    Dependency,
    /** External dependency of [STDLIB] or other [STDLIB_DEPENDENCY] package. */
    StdlibDependency,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustFeature {
    /** Name of the feature. */
    pub name: String,
    /** Feature's dependencies. */
    pub dependencies: Vec<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustCfgOptions {
    /** `cfgs` in Rust can take one of two forms: "cfg1" or "cfg2=\"string\"".
    The `cfg` is split by '=' delimiter and the first half becomes key and
    the second is aggregated to the value in `keyValueOptions`. */
    pub key_value_options: HashMap<String, Vec<String>>,
    /** A sequence of first halves after splitting `cfgs` by '='. */
    pub name_options: Vec<String>,
}

/** A `crate` is the smallest amount of code that the Rust compiler considers at a time.
It can come in one of two forms: a binary crate or a library crate.
`Binary crates` are programs you can compile to an executable that you can run,
such as a command-line program or a server.
Each must have a function called main that defines what happens when the executable runs.
`Library crates` don’t have a main function, and they don’t compile to an executable.
Instead, they define functionality intended to be shared with multiple projects.

A `package` is a bundle of one or more crates that provides a set of functionality.
It contains a Cargo.toml file that describes how to build those crates.
A package can contain many binary crates, but at most only one library crate.
However, it must contain at least one crate, whether that’s a library or binary crate. */
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustPackage {
    /** The package’s unique identifier. */
    pub id: String,
    /** The package's root path. */
    pub root_url: Uri,
    /** The name of the package. */
    pub name: String,
    /** The version of the package. */
    pub version: String,
    /** Defines a reason a package is in a project. */
    pub origin: RustPackageOrigin,
    /** Code edition of the package. */
    pub edition: Edition,
    /** The source ID of the dependency, `null` for the root package and path dependencies. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /** Correspond to source files which can be compiled into a crate from this package.
    Contains only resolved targets without conflicts. */
    pub targets: Vec<RustBuildTarget>,
    /** Same as `targets`, but contains all targets from this package.
    `targets` should be the subset of `allTargets`. */
    pub all_targets: Vec<RustBuildTarget>,
    /** Set of features defined for the package (including optional dependencies).
    Each feature maps to an array of features or dependencies it enables.
    The entry named "default" defines which features are enabled by default. */
    pub features: Vec<RustFeature>,
    /** Array of features enabled on this package. */
    pub enabled_features: Vec<String>,
    /** Conditional compilation flags that can be set based on certain conditions.
    They can be used to enable or disable certain sections of code during the build process. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_options: Option<RustCfgOptions>,
    /** Environment variables for the package. */
    pub env: HashMap<String, String>,
    /** An absolute path which is used as a value of `OUT_DIR` environmental
    variable when compiling current package. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_dir_url: Option<Uri>,
    /** File path to compiled output of a procedural macro crate.
    Procedural macros are macros that generate code at compile time.
    Contains files with file extensions: `.dll`, `.so` or `.dylib`. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proc_macro_artifact: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDepKindInfo {
    /** The dependency kind. */
    pub kind: RustDepKind,
    /** The target platform for the dependency. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Serialize_repr, Deserialize_repr, Default)]
#[repr(u8)]
pub enum RustDepKind {
    /** For old Cargo versions prior to `1.41.0`. */
    Unclassified = 1,
    /** For [dependencies]. */
    #[default]
    Normal = 2,
    /** For [dev-dependencies]. */
    Dev = 3,
    /** For [build-dependencies]. */
    Build = 4,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustDependency {
    /** The Package ID of the dependency. */
    pub pkg: String,
    /** The name of the dependency's library target.
    If this is a renamed dependency, this is the new name. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /** Array of dependency kinds. */
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
                vec![RustRawDependency::default()],
            )]),
            dependencies: HashMap::from([(
                "package_id".to_string(),
                vec![RustDependency::default()],
            )]),
            resolved_targets: vec![BuildTargetIdentifier::default()],
        };

        assert_json_snapshot!(result, @r#"
        {
          "packages": [
            {
              "id": "",
              "rootUrl": "",
              "name": "",
              "version": "",
              "origin": "workspace",
              "edition": "",
              "targets": [],
              "allTargets": [],
              "features": [],
              "enabledFeatures": [],
              "env": {}
            }
          ],
          "rawDependencies": {
            "package_id": [
              {
                "name": "",
                "optional": false,
                "usesDefaultFeatures": false,
                "features": []
              }
            ]
          },
          "dependencies": {
            "package_id": [
              {
                "pkg": "",
                "depKinds": []
              }
            ]
          },
          "resolvedTargets": [
            {
              "uri": ""
            }
          ]
        }
        "#);

        assert_json_snapshot!(RustWorkspaceParams::default(), @r#"
        {
          "targets": []
        }
        "#);
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
            features: BTreeSet::from(["test_feature".to_string()]),
        };

        assert_json_snapshot!(dependency, @r#"
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
        "#);

        assert_json_snapshot!(RustRawDependency::default(), @r#"
        {
          "name": "",
          "optional": false,
          "usesDefaultFeatures": false,
          "features": []
        }
        "#);
    }

    #[test]
    fn rust_target() {
        let target = RustBuildTarget {
            name: "test_name".to_string(),
            crate_root_url: "test_crate_url".to_string(),
            kind: RustTargetKind::default(),
            crate_types: vec![RustCrateType::default()],
            edition: Edition::default(),
            doctest: false,
            required_features: BTreeSet::from(["test_feature".to_string()]),
        };

        assert_json_snapshot!(target, @r#"
        {
          "name": "test_name",
          "crateRootUrl": "test_crate_url",
          "kind": 1,
          "crateTypes": [
            2
          ],
          "edition": "",
          "doctest": false,
          "requiredFeatures": [
            "test_feature"
          ]
        }
        "#);

        assert_json_snapshot!(RustBuildTarget::default(), @r#"
        {
          "name": "",
          "crateRootUrl": "",
          "kind": 1,
          "crateTypes": [],
          "edition": "",
          "doctest": false,
          "requiredFeatures": []
        }
        "#);
    }

    #[test]
    fn rust_feature() {
        let feature = RustFeature {
            name: "test_name".to_string(),
            dependencies: vec!["test_feature".to_string()],
        };

        assert_json_snapshot!(feature, @r#"
        {
          "name": "test_name",
          "dependencies": [
            "test_feature"
          ]
        }
        "#);

        assert_json_snapshot!(RustFeature::default(), @r#"
        {
          "name": "",
          "dependencies": []
        }
        "#);
    }

    #[test]
    fn rust_cfg_options() {
        let cfg_options = RustCfgOptions {
            key_value_options: HashMap::from([("key".to_string(), vec!["value".to_string()])]),
            name_options: vec!["name1".to_string(), "name2".to_string()],
        };

        assert_json_snapshot!(cfg_options, @r#"
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
        "#);

        assert_json_snapshot!(RustCfgOptions::default(), @r#"
        {
          "keyValueOptions": {},
          "nameOptions": []
        }
        "#);
    }

    #[test]
    fn rust_package() {
        let package = RustPackage {
            id: "test_id".to_string(),
            root_url: "test_root_url".to_string(),
            name: "test_name".to_string(),
            version: "test_version".to_string(),
            origin: RustPackageOrigin::default(),
            edition: Edition::default(),
            source: Some("test_source".to_string()),
            targets: vec![RustBuildTarget::default()],
            all_targets: vec![RustBuildTarget::default()],
            features: vec![RustFeature::default()],
            enabled_features: vec!["test_feature".to_string()],
            cfg_options: Some(RustCfgOptions::default()),
            env: HashMap::from([("key".to_string(), "value".to_string())]),
            out_dir_url: Some("test_out_dir_url".to_string()),
            proc_macro_artifact: Some(Uri::default()),
        };

        assert_json_snapshot!(package, @r#"
        {
          "id": "test_id",
          "rootUrl": "test_root_url",
          "name": "test_name",
          "version": "test_version",
          "origin": "workspace",
          "edition": "",
          "source": "test_source",
          "targets": [
            {
              "name": "",
              "crateRootUrl": "",
              "kind": 1,
              "crateTypes": [],
              "edition": "",
              "doctest": false,
              "requiredFeatures": []
            }
          ],
          "allTargets": [
            {
              "name": "",
              "crateRootUrl": "",
              "kind": 1,
              "crateTypes": [],
              "edition": "",
              "doctest": false,
              "requiredFeatures": []
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
          "cfgOptions": {
            "keyValueOptions": {},
            "nameOptions": []
          },
          "env": {
            "key": "value"
          },
          "outDirUrl": "test_out_dir_url",
          "procMacroArtifact": ""
        }
        "#);

        assert_json_snapshot!(RustPackage::default(), @r#"
        {
          "id": "",
          "rootUrl": "",
          "name": "",
          "version": "",
          "origin": "workspace",
          "edition": "",
          "targets": [],
          "allTargets": [],
          "features": [],
          "enabledFeatures": [],
          "env": {}
        }
        "#);
    }

    #[test]
    fn rust_dep_kind_info() {
        let dep_kind_info = RustDepKindInfo {
            kind: RustDepKind::default(),
            target: Some("test_target".to_string()),
        };

        assert_json_snapshot!(dep_kind_info, @r#"
        {
          "kind": 2,
          "target": "test_target"
        }
        "#);

        assert_json_snapshot!(RustDepKindInfo::default(), @r#"
        {
          "kind": 2
        }
        "#);
    }

    #[test]
    fn rust_dependency() {
        let dependency = RustDependency {
            name: Some("test_name".to_string()),
            pkg: "test_target".to_string(),
            dep_kinds: vec![RustDepKindInfo::default()],
        };

        assert_json_snapshot!(dependency, @r#"
        {
          "pkg": "test_target",
          "name": "test_name",
          "depKinds": [
            {
              "kind": 2
            }
          ]
        }
        "#);

        assert_json_snapshot!(RustDependency::default(), @r#"
        {
          "pkg": "",
          "depKinds": []
        }
        "#);
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
        assert_json_snapshot!(RustDepKind::Normal, @"2");
        assert_json_snapshot!(RustDepKind::Dev, @"3");
        assert_json_snapshot!(RustDepKind::Build, @"4");
    }

    #[test]
    fn rust_package_origin() {
        assert_json_snapshot!(RustPackageOrigin::Stdlib, @r###""stdlib""###);
        assert_json_snapshot!(RustPackageOrigin::Workspace, @r###""workspace""###);
        assert_json_snapshot!(RustPackageOrigin::Dependency, @r###""dependency""###);
        assert_json_snapshot!(RustPackageOrigin::StdlibDependency, @r###""stdlib-dependency""###);
    }

    #[test]
    fn rust_crate_type() {
        assert_json_snapshot!(RustCrateType::Bin, @"1");
        assert_json_snapshot!(RustCrateType::Lib, @"2");
        assert_json_snapshot!(RustCrateType::Rlib, @"3");
        assert_json_snapshot!(RustCrateType::Dylib, @"4");
        assert_json_snapshot!(RustCrateType::Cdylib, @"5");
        assert_json_snapshot!(RustCrateType::Staticlib, @"6");
        assert_json_snapshot!(RustCrateType::ProcMacro, @"7");
        assert_json_snapshot!(RustCrateType::Unknown, @"8");
    }
}
