use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::extensions::{Feature, FeatureDependencyGraph, RustEdition};
use crate::requests::Request;
use crate::{BuildTargetIdentifier, EnvironmentVariables, URI};

#[derive(Debug)]
pub enum RustWorkspace {}

/// The Rust workspace request is sent from the client to the server to query for
/// the information about project's workspace for the given list of build targets.
///
/// The request is essential to connect and work with `intellij-rust` plugin.
///
/// The request may take a long time, as it may require building a project to some extent
/// (for example with `cargo check` command).
impl Request for RustWorkspace {
    type Params = RustWorkspaceParams;
    type Result = RustWorkspaceResult;
    const METHOD: &'static str = "buildTarget/rustWorkspace";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceParams {
    /// A sequence of build targets for workspace resolution.
    pub targets: Vec<BuildTargetIdentifier>,
}

/// The RustRawDependencies is a mapping between
/// package id and the package's raw dependencies info.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustRawDependencies(pub BTreeMap<String, Vec<RustRawDependency>>);

impl std::ops::Deref for RustRawDependencies {
    type Target = BTreeMap<String, Vec<RustRawDependency>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<String, Vec<RustRawDependency>>> for RustRawDependencies {
    fn from(input: BTreeMap<String, Vec<RustRawDependency>>) -> Self {
        Self(input)
    }
}

/// The RustDependencies is a mapping between
/// package id and the package's dependencies info.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustDependencies(pub BTreeMap<String, Vec<RustDependency>>);

impl std::ops::Deref for RustDependencies {
    type Target = BTreeMap<String, Vec<RustDependency>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<String, Vec<RustDependency>>> for RustDependencies {
    fn from(input: BTreeMap<String, Vec<RustDependency>>) -> Self {
        Self(input)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustWorkspaceResult {
    /// Packages of given targets.
    pub packages: Vec<RustPackage>,
    /// Dependencies in `cargo metadata` as listed in the package `Cargo.toml`,
    /// without package resolution or any additional data.
    pub raw_dependencies: RustRawDependencies,
    /// Resolved dependencies of the build. Handles renamed dependencies.
    /// Correspond to dependencies from resolved dependency graph from `cargo metadata` that shows
    /// the actual dependencies that are being used in the build.
    pub dependencies: RustDependencies,
    /// A sequence of build targets taken into consideration during build process.
    pub resolved_targets: Vec<BuildTargetIdentifier>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustRawDependency {
    /// The name of the dependency.
    pub name: String,
    /// Name to which this dependency is renamed when declared in Cargo.toml.
    /// This field allows to specify an alternative name for a dependency to use in a code,
    /// regardless of how it’s published (helpful for example if multiple dependencies
    /// have conflicting names).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rename: Option<String>,
    /// The dependency kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<RustDepKind>,
    /// The target platform for the dependency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Indicates whether this is an optional dependency.
    pub optional: bool,
    /// Indicates whether default features are enabled.
    pub uses_default_features: bool,
    /// A sequence of enabled features.
    pub features: BTreeSet<Feature>,
}

/// `RustBuildTarget` is a basic data structure that contains rust-specific
/// metadata for compiling a target containing Rust sources.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustBuildTarget {
    /// The name of the target.
    pub name: String,
    /// Path to the root module of the crate.
    pub crate_root_url: URI,
    /// A target's kind.
    pub kind: RustTargetKind,
    /// Type of output that is produced by a crate during the build process.
    /// The crate type determines how the source code is compiled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crate_types: Option<Vec<RustCrateType>>,
    /// The Rust edition of the target.
    pub edition: RustEdition,
    /// Whether or not this target has doc tests enabled, and
    /// the target is compatible with doc testing.
    pub doctest: bool,
    /// A sequence of required features.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_features: Option<BTreeSet<Feature>>,
}

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum RustTargetKind {
    #[default]
    /// For lib targets.
    Lib = 1,
    /// For binaries.
    Bin = 2,
    /// For integration tests.
    Test = 3,
    /// For examples.
    Example = 4,
    /// For benchmarks.
    Bench = 5,
    /// For build scripts.
    CustomBuild = 6,
    /// For unknown targets.
    Unknown = 7,
}

/// Crate types (`lib`, `rlib`, `dylib`, `cdylib`, `staticlib`) are listed for
/// `lib` and `example` target kinds. For other target kinds `bin` crate type is listed.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum RustCrateType {
    #[default]
    Bin = 1,
    Lib = 2,
    Rlib = 3,
    Dylib = 4,
    Cdylib = 5,
    Staticlib = 6,
    ProcMacro = 7,
    Unknown = 8,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustPackageOrigin(pub std::borrow::Cow<'static, str>);

impl RustPackageOrigin {
    /// External dependency of [WORKSPACE] or other [DEPENDENCY] package.
    pub const DEPENDENCY: RustPackageOrigin = RustPackageOrigin::new("dependency");
    /// The package comes from the standard library.
    pub const STDLIB: RustPackageOrigin = RustPackageOrigin::new("stdlib");
    /// External dependency of [STDLIB] or other [STDLIB_DEPENDENCY] package.
    pub const STDLIB_DEPENDENCY: RustPackageOrigin = RustPackageOrigin::new("stdlib-dependency");
    /// The package is a part of our workspace.
    pub const WORKSPACE: RustPackageOrigin = RustPackageOrigin::new("workspace");

    pub const fn new(tag: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(tag))
    }
}

/// A `crate` is the smallest amount of code that the Rust compiler considers at a time.
/// It can come in one of two forms: a binary crate or a library crate.
/// `Binary crates` are programs you can compile to an executable that you can run,
/// such as a command-line program or a server.
/// Each must have a function called main that defines what happens when the executable runs.
/// `Library crates` don’t have a main function, and they don’t compile to an executable.
/// Instead, they define functionality intended to be shared with multiple projects.
///
/// A `package` is a bundle of one or more crates that provides a set of functionality.
/// It contains a Cargo.toml file that describes how to build those crates.
/// A package can contain many binary crates, but at most only one library crate.
/// However, it must contain at least one crate, whether that’s a library or binary crate.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustPackage {
    /// The package’s unique identifier
    pub id: String,
    /// The package's root path.
    pub root_url: URI,
    /// The name of the package.
    pub name: String,
    /// The version of the package.
    pub version: String,
    /// Defines a reason a package is in a project.
    pub origin: RustPackageOrigin,
    /// Code edition of the package.
    pub edition: RustEdition,
    /// The source ID of the dependency, for example:
    /// "registry+https://github.com/rust-lang/crates.io-index".
    /// `null` for the root package and path dependencies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Corresponds to source files which can be compiled into a crate from this package.
    /// Contains only resolved targets without conflicts.
    pub resolved_targets: Vec<RustBuildTarget>,
    /// Same as `resolvedTargets`, but contains all targets from this package.
    /// `targets` should be the subset of `allTargets`.
    pub all_targets: Vec<RustBuildTarget>,
    /// Set of features defined for the package (including optional dependencies).
    /// Each feature maps to an array of features or dependencies it enables.
    /// The entry named "default" defines which features are enabled by default.
    pub features: FeatureDependencyGraph,
    /// Array of features enabled on this package.
    pub enabled_features: BTreeSet<Feature>,
    /// Conditional compilation flags that can be set based on certain conditions.
    /// They can be used to enable or disable certain sections of code during the build process.
    /// `cfgs` in Rust can take one of two forms: "cfg1" or "cfg2=\"string\"".
    /// The `cfg` is split by '=' delimiter and the first half becomes key and
    /// the second is aggregated to the value in `RustCfgOptions`.
    /// For "cfg1" the value is empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfg_options: Option<RustCfgOptions>,
    /// Environment variables for the package.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<EnvironmentVariables>,
    /// An absolute path which is used as a value of `OUT_DIR` environmental
    /// variable when compiling current package.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub out_dir_url: Option<URI>,
    /// File path to compiled output of a procedural macro crate.
    /// Procedural macros are macros that generate code at compile time.
    /// Contains files with file extensions: `.dll`, `.so` or `.dylib`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proc_macro_artifact: Option<URI>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustDepKindInfo {
    /// The dependency kind.
    pub kind: RustDepKind,
    /// The target platform for the dependency.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustDepKind(pub std::borrow::Cow<'static, str>);

impl RustDepKind {
    /// For [build-dependencies].
    pub const BUILD: RustDepKind = RustDepKind::new("build");
    /// For [dev-dependencies].
    pub const DEV: RustDepKind = RustDepKind::new("dev");
    /// For [dependencies].
    pub const NORMAL: RustDepKind = RustDepKind::new("normal");
    /// For old Cargo versions prior to `1.41.0`.
    pub const UNCLASSIFIED: RustDepKind = RustDepKind::new("unclassified");

    pub const fn new(tag: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(tag))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustDependency {
    /// The Package ID of the dependency.
    pub pkg: String,
    /// The name of the dependency's library target.
    /// If this is a renamed dependency, this is the new name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Array of dependency kinds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dep_kinds: Option<Vec<RustDepKindInfo>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustCfgOptions(pub BTreeMap<String, Vec<String>>);

impl std::ops::Deref for RustCfgOptions {
    type Target = BTreeMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<String, Vec<String>>> for RustCfgOptions {
    fn from(input: BTreeMap<String, Vec<String>>) -> Self {
        Self(input)
    }
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
            raw_dependencies: BTreeMap::from([(
                "package_id".to_string(),
                vec![RustRawDependency::default()],
            )])
            .into(),
            dependencies: BTreeMap::from([(
                "package_id".to_string(),
                vec![RustDependency::default()],
            )])
            .into(),
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
              "origin": "",
              "edition": "",
              "resolvedTargets": [],
              "allTargets": [],
              "features": {},
              "enabledFeatures": []
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
                "pkg": ""
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
            kind: Some(RustDepKind::NORMAL),
            target: Some("test_target".to_string()),
            optional: false,
            uses_default_features: false,
            features: BTreeSet::from(["test_feature".into()]),
        };

        assert_json_snapshot!(dependency, @r#"
        {
          "name": "test_name",
          "rename": "test_rename",
          "kind": "normal",
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
            crate_root_url: "test_crate_url".into(),
            kind: RustTargetKind::default(),
            crate_types: Some(vec![RustCrateType::default()]),
            edition: RustEdition::default(),
            doctest: false,
            required_features: Some(BTreeSet::from(["test_feature".into()])),
        };

        assert_json_snapshot!(target, @r#"
        {
          "name": "test_name",
          "crateRootUrl": "test_crate_url",
          "kind": 1,
          "crateTypes": [
            1
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
          "edition": "",
          "doctest": false
        }
        "#);
    }

    #[test]
    fn rust_package() {
        let package = RustPackage {
            id: "test_id".to_string(),
            root_url: "test_root_url".into(),
            name: "test_name".to_string(),
            version: "test_version".to_string(),
            origin: RustPackageOrigin::WORKSPACE,
            edition: RustEdition::default(),
            source: Some("test_source".to_string()),
            resolved_targets: vec![RustBuildTarget::default()],
            all_targets: vec![RustBuildTarget::default()],
            features: BTreeMap::from([(
                Feature::from("test_feature"),
                BTreeSet::from([Feature::from("test_feature_dependency")]),
            )])
            .into(),
            enabled_features: BTreeSet::from(["test_feature".into()]),
            cfg_options: Some(
                BTreeMap::from([("test_cfg".to_string(), vec!["test_option".to_string()])]).into(),
            ),
            env: Some(BTreeMap::from([("key".to_string(), "value".to_string())]).into()),
            out_dir_url: Some("test_out_dir_url".into()),
            proc_macro_artifact: Some(URI::default()),
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
          "resolvedTargets": [
            {
              "name": "",
              "crateRootUrl": "",
              "kind": 1,
              "edition": "",
              "doctest": false
            }
          ],
          "allTargets": [
            {
              "name": "",
              "crateRootUrl": "",
              "kind": 1,
              "edition": "",
              "doctest": false
            }
          ],
          "features": {
            "test_feature": [
              "test_feature_dependency"
            ]
          },
          "enabledFeatures": [
            "test_feature"
          ],
          "cfgOptions": {
            "test_cfg": [
              "test_option"
            ]
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
          "origin": "",
          "edition": "",
          "resolvedTargets": [],
          "allTargets": [],
          "features": {},
          "enabledFeatures": []
        }
        "#);
    }

    #[test]
    fn rust_dep_kind_info() {
        let dep_kind_info = RustDepKindInfo {
            kind: RustDepKind::NORMAL,
            target: Some("test_target".to_string()),
        };

        assert_json_snapshot!(dep_kind_info, @r#"
        {
          "kind": "normal",
          "target": "test_target"
        }
        "#);

        assert_json_snapshot!(RustDepKindInfo::default(), @r#"
        {
          "kind": ""
        }
        "#);
    }

    #[test]
    fn rust_dependency() {
        let dependency = RustDependency {
            name: Some("test_name".to_string()),
            pkg: "test_target".to_string(),
            dep_kinds: Some(vec![RustDepKindInfo::default()]),
        };

        assert_json_snapshot!(dependency, @r#"
        {
          "pkg": "test_target",
          "name": "test_name",
          "depKinds": [
            {
              "kind": ""
            }
          ]
        }
        "#);

        assert_json_snapshot!(RustDependency::default(), @r#"
        {
          "pkg": ""
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
        assert_json_snapshot!(RustDepKind::UNCLASSIFIED, @r#""unclassified""#);
        assert_json_snapshot!(RustDepKind::NORMAL, @r#""normal""#);
        assert_json_snapshot!(RustDepKind::DEV, @r#""dev""#);
        assert_json_snapshot!(RustDepKind::BUILD, @r#""build""#);
    }

    #[test]
    fn rust_package_origin() {
        assert_json_snapshot!(RustPackageOrigin::STDLIB, @r#""stdlib""#);
        assert_json_snapshot!(RustPackageOrigin::WORKSPACE, @r#""workspace""#);
        assert_json_snapshot!(RustPackageOrigin::DEPENDENCY, @r#""dependency""#);
        assert_json_snapshot!(RustPackageOrigin::STDLIB_DEPENDENCY, @r#""stdlib-dependency""#);
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
