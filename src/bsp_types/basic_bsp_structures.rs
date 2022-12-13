use serde::{Deserialize, Serialize};
use serde_json::Value;

/**  A resource identifier that is a valid URI according
* to rfc3986: * https://tools.ietf.org/html/rfc3986 */
pub type Uri = String; //dev: lsp_types uses url crate

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentIdentifier {
    pub uri: Uri,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildTarget {
    /** The target’s unique identifier */
    id: BuildTargetIdentifier,

    /** A human readable name for this target.
     * May be presented in the user interface.
     * Should be unique if possible.
     * The id.uri is used if None. */
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,

    /** The directory where this target belongs to. Multiple build targets are allowed to map
     * to the same base directory, and a build target is not required to have a base directory.
     * A base directory does not determine the sources of a target, see buildTarget/sources. */
    #[serde(skip_serializing_if = "Option::is_none")]
    base_directory: Option<Uri>,

    /** Free-form string tags to categorize or label this build target.
     * For example, can be used by the client to:
     * - customize how the target should be translated into the client's project model.
     * - group together different but related targets in the user interface.
     * - display icons or colors in the user interface.
     * Pre-defined tags are listed in `build_target_tag` but clients and servers
     * are free to define new tags for custom purposes. */
    tags: Vec<String>,

    /** The capabilities of this build target. */
    capabilities: BuildTargetCapabilities,

    /** The set of languages that this target contains.
     * The ID string for each language is defined in the LSP. */
    language_ids: Vec<String>,

    /** The direct upstream build target dependencies of this build target */
    dependencies: Vec<BuildTargetIdentifier>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    data_kind: Option<String>,

    /** Language-specific metadata about this target.
     * See ScalaBuildTarget as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/** A unique identifier for a target, can use any URI-compatible encoding as long as it is unique
* within the workspace. Clients should not infer metadata out of the URI structure such as the path
* or query parameters, use BuildTarget instead.*/
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetIdentifier {
    /** The target’s Uri */
    uri: Uri,
}

pub mod build_target_data_kind {
    /** The `data` field contains a `RustBuildTarget` object. */
    pub const RUST: &str = "rust";

    /** The `data` field contains a `CargoBuildTarget` object. */
    pub const CARGO: &str = "cargo";
}

pub mod build_target_tag {
    /** Target contains re-usable functionality for downstream targets. May have any
     * combination of capabilities. */
    pub const LIBRARY: &str = "library";

    /** Target contains source code for producing any kind of application, may have
     * but does not require the `canRun` capability. */
    pub const APPLICATION: &str = "application";

    /** Target contains source code for testing purposes, may have but does not
     * require the `canTest` capability. */
    pub const TEST: &str = "test";

    /** Target contains source code for integration testing purposes, may have
     * but does not require the `canTest` capability.
     * The difference between "test" and "integration-test" is that
     * integration tests traditionally run slower compared to normal tests
     * and require more computing resources to execute. */
    pub const INTEGRATION_TEST: &str = "integration-test";
    /** Target contains source code to measure performance of a program, may have
     * but does not require the `canRun` build target capability. */
    pub const BENCHMARK: &str = "benchmark";

    /** Target should be ignored by IDEs. */
    pub const NO_IDE: &str = "no-ide";

    /** Actions on the target such as build and test should only be invoked manually
     * and explicitly. For example, triggering a build on all targets in the workspace
     * should by default not include this target.
     *
     * The original motivation to add the "manual" tag comes from a similar functionality
     * that exists in Bazel, where targets with this tag have to be specified explicitly
     * on the command line. */
    pub const MANUAL: &str = "manual";
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetCapabilities {
    /** This target can be compiled by the BSP server. */
    pub can_compile: bool,
    /** This target can be tested by the BSP server. */
    pub can_test: bool,
    /** This target can be run by the BSP server. */
    pub can_run: bool,
    /** This target can be debugged by the BSP server. */
    pub can_debug: bool,
}
