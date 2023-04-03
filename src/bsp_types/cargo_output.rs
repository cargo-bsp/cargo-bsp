use cargo_metadata::camino::Utf8PathBuf;
use cargo_platform::Platform;
use semver::{Version, VersionReq};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Eq, PartialEq, Clone, Debug, Copy, Hash, Serialize, Deserialize, Default)]
/// Dependencies can come in three kinds
pub enum DependencyKind {
    #[default]
    #[serde(rename = "normal")]
    /// The 'normal' kind
    Normal,
    #[serde(rename = "dev")]
    /// Those used in tests only
    Development,
    #[serde(rename = "build")]
    /// Those used in build scripts only
    Build,
    #[doc(hidden)]
    #[serde(other)]
    Unknown,
}

/// The `kind` can be `null`, which is interpreted as the default - `Normal`.
pub(super) fn parse_dependency_kind<'de, D>(d: D) -> Result<DependencyKind, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
/// A dependency of the main crate
pub struct Dependency {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    /// The source of dependency
    pub source: Option<String>,
    /// The required version
    pub req: VersionReq,
    /// The kind of dependency this is
    #[serde(deserialize_with = "parse_dependency_kind")]
    pub kind: DependencyKind,
    /// Whether this dependency is required or optional
    pub optional: bool,
    /// Whether the default features in this dependency are used.
    pub uses_default_features: bool,
    /// The list of features enabled for this dependency.
    pub features: Vec<String>,
    /// The target this dependency is specific to.
    ///
    /// Use the [`Display`] trait to access the contents.
    ///
    /// [`Display`]: std::fmt::Display
    pub target: Option<Platform>,
    /// If the dependency is renamed, this is the new name for the dependency
    /// as a string.  None if it is not renamed.
    pub rename: Option<String>,
    /// The URL of the index of the registry where this dependency is from.
    ///
    /// If None, the dependency is from crates.io.
    pub registry: Option<String>,
    /// The file system path for a local path dependency.
    ///
    /// Only produced on cargo 1.51+
    pub path: Option<Utf8PathBuf>,
}

/// The error code associated to this diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct DiagnosticCode {
    /// The code itself.
    pub code: String,
    /// An explanation for the code
    pub explanation: Option<String>,
}

/// A line of code associated with the Diagnostic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct DiagnosticSpanLine {
    /// The line of code associated with the error
    pub text: String,
    /// Start of the section of the line to highlight. 1-based, character offset in self.text
    pub highlight_start: usize,
    /// End of the section of the line to highlight. 1-based, character offset in self.text
    pub highlight_end: usize,
}

/// Macro expansion information associated with a diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct DiagnosticSpanMacroExpansion {
    /// span where macro was applied to generate this code; note that
    /// this may itself derive from a macro (if
    /// `span.expansion.is_some()`)
    pub span: DiagnosticSpan,

    /// name of macro that was applied (e.g., "foo!" or "#[derive(Eq)]")
    pub macro_decl_name: String,

    /// span where macro was defined (if known)
    pub def_site_span: Option<DiagnosticSpan>,
}

/// A section of the source code associated with a Diagnostic
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct DiagnosticSpan {
    /// The file name or the macro name this diagnostic comes from.
    pub file_name: String,
    /// The byte offset in the file where this diagnostic starts from.
    pub byte_start: u32,
    /// The byte offset in the file where this diagnostic ends.
    pub byte_end: u32,
    /// 1-based. The line in the file.
    pub line_start: usize,
    /// 1-based. The line in the file.
    pub line_end: usize,
    /// 1-based, character offset.
    pub column_start: usize,
    /// 1-based, character offset.
    pub column_end: usize,
    /// Is this a "primary" span -- meaning the point, or one of the points,
    /// where the error occurred?
    ///
    /// There are rare cases where multiple spans are marked as primary,
    /// e.g. "immutable borrow occurs here" and "mutable borrow ends here" can
    /// be two separate spans both "primary". Top (parent) messages should
    /// always have at least one primary span, unless it has 0 spans. Child
    /// messages may have 0 or more primary spans.
    pub is_primary: bool,
    /// Source text from the start of line_start to the end of line_end.
    pub text: Vec<DiagnosticSpanLine>,
    /// Label that should be placed at this location (if any)
    pub label: Option<String>,
    /// If we are suggesting a replacement, this will contain text
    /// that should be sliced in atop this span.
    pub suggested_replacement: Option<String>,
    /// If the suggestion is approximate
    pub suggestion_applicability: Option<Applicability>,
    /// Macro invocations that created the code at this span, if any.
    pub expansion: Option<Box<DiagnosticSpanMacroExpansion>>,
}

/// Whether a suggestion can be safely applied.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Applicability {
    /// The suggested replacement can be applied automatically safely
    MachineApplicable,
    /// The suggested replacement has placeholders that will need to be manually
    /// replaced.
    HasPlaceholders,
    /// The suggested replacement may be incorrect in some circumstances. Needs
    /// human review.
    MaybeIncorrect,
    /// The suggested replacement will probably not work.
    Unspecified,
}

/// The diagnostic level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    /// Internal compiler error
    #[serde(rename = "error: internal compiler error")]
    Ice,
    /// Error
    Error,
    /// Warning
    Warning,
    /// Failure note
    #[serde(rename = "failure-note")]
    FailureNote,
    /// Note
    Note,
    /// Help
    Help,
}

/// A diagnostic message generated by rustc
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct Diagnostic {
    /// The error message of this diagnostic.
    pub message: String,
    /// The associated error code for this diagnostic
    pub code: Option<DiagnosticCode>,
    /// "error: internal compiler error", "error", "warning", "note", "help"
    pub level: DiagnosticLevel,
    /// A list of source code spans this diagnostic is associated with.
    pub spans: Vec<DiagnosticSpan>,
    /// Associated diagnostic messages.
    pub children: Vec<Diagnostic>,
    /// The message as rustc would render it
    pub rendered: Option<String>,
}

/// An "opaque" identifier for a package.
/// It is possible to inspect the `repr` field, if the need arises, but its
/// precise format is an implementation detail and is subject to change.
///
/// `Metadata` can be indexed by `PackageId`.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct PackageId {
    /// The underlying string representation of id.
    pub repr: String,
}

/// Helpers for default metadata fields
fn is_null(value: &serde_json::Value) -> bool {
    matches!(value, serde_json::Value::Null)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// Starting point for metadata returned by `cargo metadata`
pub struct Metadata {
    /// A list of all crates referenced by this crate (and the crate itself)
    pub packages: Vec<Package>,
    /// A list of all workspace members
    pub workspace_members: Vec<PackageId>,
    /// Dependencies graph
    pub resolve: Option<Resolve>,
    /// Workspace root
    pub workspace_root: Utf8PathBuf,
    /// Build directory
    pub target_directory: Utf8PathBuf,
    /// The workspace-level metadata object. Null if non-existent.
    #[serde(rename = "metadata", default, skip_serializing_if = "is_null")]
    pub workspace_metadata: serde_json::Value,
    /// The metadata format version
    version: usize,
}

impl Metadata {
    /// Get the root package of this metadata instance.
    pub fn root_package(&self) -> Option<&Package> {
        let root = self.resolve.as_ref()?.root.as_ref()?;
        self.packages.iter().find(|pkg| &pkg.id == root)
    }

    /// Get the workspace packages.
    pub fn workspace_packages(&self) -> Vec<&Package> {
        self.packages
            .iter()
            .filter(|&p| self.workspace_members.contains(&p.id))
            .collect()
    }
}

impl<'a> std::ops::Index<&'a PackageId> for Metadata {
    type Output = Package;

    fn index(&self, idx: &'a PackageId) -> &Package {
        self.packages
            .iter()
            .find(|p| p.id == *idx)
            .unwrap_or_else(|| panic!("no package with this id: {:?}", idx))
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// A dependency graph
pub struct Resolve {
    /// Nodes in a dependencies graph
    pub nodes: Vec<Node>,

    /// The crate for which the metadata was read.
    pub root: Option<PackageId>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// A node in a dependencies graph
pub struct Node {
    /// An opaque identifier for a package
    pub id: PackageId,
    /// Dependencies in a structured format.
    ///
    /// `deps` handles renamed dependencies whereas `dependencies` does not.
    #[serde(default)]
    pub deps: Vec<NodeDep>,

    /// List of opaque identifiers for this node's dependencies.
    /// It doesn't support renamed dependencies. See `deps`.
    pub dependencies: Vec<PackageId>,

    /// Features enabled on the crate
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// A dependency in a node
pub struct NodeDep {
    /// The name of the dependency's library target.
    /// If the crate was renamed, it is the new name.
    pub name: String,
    /// Package ID (opaque unique identifier)
    pub pkg: PackageId,
    /// The kinds of dependencies.
    ///
    /// This field was added in Rust 1.41.
    #[serde(default)]
    pub dep_kinds: Vec<DepKindInfo>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// Information about a dependency kind.
pub struct DepKindInfo {
    /// The kind of dependency.
    #[serde(deserialize_with = "parse_dependency_kind")]
    pub kind: DependencyKind,
    /// The target platform for the dependency.
    ///
    /// This is `None` if it is not a target dependency.
    ///
    /// Use the [`Display`] trait to access the contents.
    ///
    /// By default all platform dependencies are included in the resolve
    /// graph. Use Cargo's `--filter-platform` flag if you only want to
    /// include dependencies for a specific platform.
    ///
    /// [`Display`]: std::fmt::Display
    pub target: Option<Platform>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// One or more crates described by a single `Cargo.toml`
///
/// Each [`target`][Package::targets] of a `Package` will be built as a crate.
/// For more information, see <https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html>.
pub struct Package {
    /// Name as given in the `Cargo.toml`
    pub name: String,
    /// Version given in the `Cargo.toml`
    pub version: Version,
    /// Authors given in the `Cargo.toml`
    #[serde(default)]
    pub authors: Vec<String>,
    /// An opaque identifier for a package
    pub id: PackageId,
    /// The source of the package, e.g.
    /// crates.io or `None` for local projects.
    pub source: Option<Source>,
    /// Description as given in the `Cargo.toml`
    pub description: Option<String>,
    /// List of dependencies of this particular package
    pub dependencies: Vec<Dependency>,
    /// License as given in the `Cargo.toml`
    pub license: Option<String>,
    /// If the package is using a nonstandard license, this key may be specified instead of
    /// `license`, and must point to a file relative to the manifest.
    pub license_file: Option<Utf8PathBuf>,
    /// Targets provided by the crate (lib, bin, example, test, ...)
    pub targets: Vec<Target>,
    /// Features provided by the crate, mapped to the features required by that feature.
    pub features: HashMap<String, Vec<String>>,
    /// Path containing the `Cargo.toml`
    pub manifest_path: Utf8PathBuf,
    /// Categories as given in the `Cargo.toml`
    #[serde(default)]
    pub categories: Vec<String>,
    /// Keywords as given in the `Cargo.toml`
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Readme as given in the `Cargo.toml`
    pub readme: Option<Utf8PathBuf>,
    /// Repository as given in the `Cargo.toml`
    // can't use `url::Url` because that requires a more recent stable compiler
    pub repository: Option<String>,
    /// Homepage as given in the `Cargo.toml`
    ///
    /// On versions of cargo before 1.49, this will always be [`None`].
    pub homepage: Option<String>,
    /// Documentation URL as given in the `Cargo.toml`
    ///
    /// On versions of cargo before 1.49, this will always be [`None`].
    pub documentation: Option<String>,
    /// Default Rust edition for the package
    ///
    /// Beware that individual targets may specify their own edition in
    /// [`Target::edition`].
    #[serde(default)]
    pub edition: Edition,
    /// Contents of the free form package.metadata section
    ///
    /// This contents can be serialized to a struct using serde:
    ///
    /// ```rust
    /// use serde::Deserialize;
    /// use serde_json::json;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct SomePackageMetadata {
    ///     some_value: i32,
    /// }
    ///
    /// fn main() {
    ///     let value = json!({
    ///         "some_value": 42,
    ///     });
    ///
    ///     let package_metadata: SomePackageMetadata = serde_json::from_value(value).unwrap();
    ///     assert_eq!(package_metadata.some_value, 42);
    /// }
    ///
    /// ```
    #[serde(default, skip_serializing_if = "is_null")]
    pub metadata: serde_json::Value,
    /// The name of a native library the package is linking to.
    pub links: Option<String>,
    /// List of registries to which this package may be published.
    ///
    /// Publishing is unrestricted if `None`, and forbidden if the `Vec` is empty.
    ///
    /// This is always `None` if running with a version of Cargo older than 1.39.
    pub publish: Option<Vec<String>>,
    /// The default binary to run by `cargo run`.
    ///
    /// This is always `None` if running with a version of Cargo older than 1.55.
    pub default_run: Option<String>,
    /// The minimum supported Rust version of this package.
    ///
    /// This is always `None` if running with a version of Cargo older than 1.58.
    pub rust_version: Option<VersionReq>,
}

impl Package {
    /// Full path to the license file if one is present in the manifest
    pub fn license_file(&self) -> Option<Utf8PathBuf> {
        self.license_file.as_ref().map(|file| {
            self.manifest_path
                .parent()
                .unwrap_or(&self.manifest_path)
                .join(file)
        })
    }

    /// Full path to the readme file if one is present in the manifest
    pub fn readme(&self) -> Option<Utf8PathBuf> {
        self.readme
            .as_ref()
            .map(|file| self.manifest_path.join(file))
    }
}

/// The source of a package such as crates.io.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(transparent)]
pub struct Source {
    /// The underlying string representation of a source.
    pub repr: String,
}

impl Source {
    /// Returns true if the source is crates.io.
    pub fn is_crates_io(&self) -> bool {
        self.repr == "registry+https://github.com/rust-lang/crates.io-index"
    }
}

impl std::fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.repr, f)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
/// A single target (lib, bin, example, ...) provided by a crate
pub struct Target {
    /// Name as given in the `Cargo.toml` or generated from the file name
    pub name: String,
    /// Kind of target ("bin", "example", "test", "bench", "lib")
    pub kind: Vec<String>,
    /// Almost the same as `kind`, except when an example is a library instead of an executable.
    /// In that case `crate_types` contains things like `rlib` and `dylib` while `kind` is `example`
    #[serde(default)]
    #[cfg_attr(feature = "builder", builder(default))]
    pub crate_types: Vec<String>,

    #[serde(default)]
    #[cfg_attr(feature = "builder", builder(default))]
    #[serde(rename = "required-features")]
    /// This target is built only if these features are enabled.
    /// It doesn't apply to `lib` targets.
    pub required_features: Vec<String>,
    /// Path to the main source file of the target
    pub src_path: Utf8PathBuf,
    /// Rust edition for this target
    #[serde(default)]
    #[cfg_attr(feature = "builder", builder(default))]
    pub edition: Edition,
    /// Whether or not this target has doc tests enabled, and the target is
    /// compatible with doc testing.
    ///
    /// This is always `true` if running with a version of Cargo older than 1.37.
    #[serde(default = "default_true")]
    #[cfg_attr(feature = "builder", builder(default = "true"))]
    pub doctest: bool,
    /// Whether or not this target is tested by default by `cargo test`.
    ///
    /// This is always `true` if running with a version of Cargo older than 1.47.
    #[serde(default = "default_true")]
    #[cfg_attr(feature = "builder", builder(default = "true"))]
    pub test: bool,
    /// Whether or not this target is documented by `cargo doc`.
    ///
    /// This is always `true` if running with a version of Cargo older than 1.50.
    #[serde(default = "default_true")]
    #[cfg_attr(feature = "builder", builder(default = "true"))]
    pub doc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// The rust edition
pub enum Edition {
    /// Edition 2015
    #[serde(rename = "2015")]
    E2015,
    /// Edition 2018
    #[serde(rename = "2018")]
    E2018,
    /// Edition 2021
    #[serde(rename = "2021")]
    E2021,
}

impl Default for Edition {
    fn default() -> Self {
        Self::E2015
    }
}

fn default_true() -> bool {
    true
}

/// Cargo features flags
#[derive(Debug, Clone)]
pub enum CargoOpt {
    /// Run cargo with `--features-all`
    AllFeatures,
    /// Run cargo with `--no-default-features`
    NoDefaultFeatures,
    /// Run cargo with `--features <FEATURES>`
    SomeFeatures(Vec<String>),
}

/// Profile settings used to determine which compiler flags to use for a
/// target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct ArtifactProfile {
    /// Optimization level. Possible values are 0-3, s or z.
    pub opt_level: String,
    /// The amount of debug info. 0 for none, 1 for limited, 2 for full
    pub debuginfo: Option<u32>,
    /// State of the `cfg(debug_assertions)` directive, enabling macros like
    /// `debug_assert!`
    pub debug_assertions: bool,
    /// State of the overflow checks.
    pub overflow_checks: bool,
    /// Whether this profile is a test
    pub test: bool,
}

/// A compiler-generated file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct Artifact {
    /// The package this artifact belongs to
    pub package_id: PackageId,
    /// The target this artifact was compiled for
    pub target: Target,
    /// The profile this artifact was compiled with
    pub profile: ArtifactProfile,
    /// The enabled features for this artifact
    pub features: Vec<String>,
    /// The full paths to the generated artifacts
    /// (e.g. binary file and separate debug info)
    pub filenames: Vec<Utf8PathBuf>,
    /// Path to the executable file
    pub executable: Option<Utf8PathBuf>,
    /// If true, then the files were already generated
    pub fresh: bool,
}

/// Message left by the compiler
// TODO: Better name. This one comes from machine_message.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct CompilerMessage {
    /// The package this message belongs to
    pub package_id: PackageId,
    /// The target this message is aimed at
    pub target: Target,
    /// The message the compiler sent.
    pub message: Diagnostic,
}

/// Output of a build script execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct BuildScript {
    /// The package this build script execution belongs to
    pub package_id: PackageId,
    /// The libs to link
    pub linked_libs: Vec<Utf8PathBuf>,
    /// The paths to search when resolving libs
    pub linked_paths: Vec<Utf8PathBuf>,
    /// Various `--cfg` flags to pass to the compiler
    pub cfgs: Vec<String>,
    /// The environment variables to add to the compilation
    pub env: Vec<(String, String)>,
    /// The `OUT_DIR` environment variable where this script places its output
    ///
    /// Added in Rust 1.41.
    #[serde(default)]
    pub out_dir: Utf8PathBuf,
}

/// Final result of a build.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "builder", derive(Builder))]
#[cfg_attr(feature = "builder", builder(pattern = "owned", setter(into)))]
pub struct BuildFinished {
    /// Whether or not the build finished successfully.
    pub success: bool,
}

/// A cargo message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum Message {
    /// The compiler generated an artifact
    CompilerArtifact(Artifact),
    /// The compiler wants to display a message
    CompilerMessage(CompilerMessage),
    /// A build script successfully executed.
    BuildScriptExecuted(BuildScript),
    /// The build has finished.
    ///
    /// This is emitted at the end of the build as the last message.
    /// Added in Rust 1.44.
    BuildFinished(BuildFinished),
    /// A line of text which isn't a cargo or compiler message.
    /// Line separator is not included
    #[serde(skip)]
    TextLine(String),
}
