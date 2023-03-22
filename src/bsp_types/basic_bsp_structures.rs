use cargo_metadata::Edition;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/**  A resource identifier that is a valid URI according
* to rfc3986: * https://tools.ietf.org/html/rfc3986 */
pub type Uri = String; //dev: lsp_types uses url crate

pub const RUST_ID: &str = "rust";

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TextDocumentIdentifier {
    pub uri: Uri,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildClientCapabilities {
    /** The languages that this client supports.
     * The ID strings for each language is defined in the LSP.
     * The server must never respond with build targets for other
     * languages than those that appear in this list. */
    pub language_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerCapabilities {
    /** The languages the server supports compilation via method buildTarget/compile. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_provider: Option<CompileProvider>,

    /** The languages the server supports test execution via method buildTarget/test */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_provider: Option<TestProvider>,

    /** The languages the server supports run via method buildTarget/run */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_provider: Option<RunProvider>,

    /** The languages the server supports debugging via method debugSession/start */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_provider: Option<DebugProvider>,

    /** The server can provide a list of targets that contain a
     * single text document via the method buildTarget/inverseSources */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse_sources_provider: Option<bool>,

    /** The server provides sources for library dependencies
     * via method buildTarget/dependencySources */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_sources_provider: Option<bool>,

    /** The server can provide a list of dependency modules (libraries with meta information)
     * via method buildTarget/dependencyModules */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_modules_provider: Option<bool>,

    /** The server provides all the resource dependencies
     * via method buildTarget/resources */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources_provider: Option<bool>,

    /** The server provides all output paths
     * via method buildTarget/outputPaths */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_paths_provider: Option<bool>,

    /** The server sends notifications to the client on build
     * target change events via buildTarget/didChange */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_target_changed_provider: Option<bool>,

    /** The server can respond to `buildTarget/jvmRunEnvironment` requests with the
     * necessary information required to launch a Java process to run a main class. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jvm_run_environment_provider: Option<bool>,

    /** The server can respond to `buildTarget/jvmTestEnvironment` requests with the
     * necessary information required to launch a Java process for testing or
     * debugging. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jvm_test_environment_provider: Option<bool>,

    /** Reloading the build state through workspace/reload is supported */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_reload: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompileProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RunProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DebugProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildTarget {
    /** The target’s unique identifier */
    pub id: BuildTargetIdentifier,

    /** A human readable name for this target.
     * May be presented in the user interface.
     * Should be unique if possible.
     * The id.uri is used if None. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /** The directory where this target belongs to. Multiple build targets are allowed to map
     * to the same base directory, and a build target is not required to have a base directory.
     * A base directory does not determine the sources of a target, see buildTarget/sources. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_directory: Option<Uri>,

    /** Free-form string tags to categorize or label this build target.
     * For example, can be used by the client to:
     * - customize how the target should be translated into the client's project model.
     * - group together different but related targets in the user interface.
     * - display icons or colors in the user interface.
     * Pre-defined tags are listed in `build_target_tag` but clients and servers
     * are free to define new tags for custom purposes. */
    pub tags: Vec<BuildTargetTag>,

    /** The capabilities of this build target. */
    pub capabilities: BuildTargetCapabilities,

    /** The set of languages that this target contains.
     * The ID string for each language is defined in the LSP. */
    pub language_ids: Vec<String>,

    /** The direct upstream build target dependencies of this build target */
    pub dependencies: Vec<BuildTargetIdentifier>,

    /** Kind of data to expect in the `data` field. If this field is not set, the kind of data is not specified. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_kind: Option<String>,

    /** Language-specific metadata about this target.
     * See ScalaBuildTarget as an example. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<RustBuildTarget>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RustBuildTarget {
    pub edition: Edition,
    pub required_features: Vec<String>,
}

/** A unique identifier for a target, can use any URI-compatible encoding as long as it is unique
* within the workspace. Clients should not infer metadata out of the URI structure such as the path
* or query parameters, use BuildTarget instead.*/
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct BuildTargetIdentifier {
    /** The target’s Uri */
    pub uri: Uri,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum BuildTargetTag {
    /** Target contains re-usable functionality for downstream targets. May have any
     * combination of capabilities. */
    Library,

    /** Target contains source code for producing any kind of application, may have
     * but does not require the `canRun` capability. */
    Application,

    /** Target contains source code for testing purposes, may have but does not
     * require the `canTest` capability. */
    Test,

    /** Target contains source code for integration testing purposes, may have
     * but does not require the `canTest` capability.
     * The difference between "test" and "integration-test" is that
     * integration tests traditionally run slower compared to normal tests
     * and require more computing resources to execute. */
    IntegrationTest,

    /** Target contains source code to measure performance of a program, may have
     * but does not require the `canRun` build target capability. */
    Benchmark,

    /** Target should be ignored by IDEs. */
    NoIde,

    /** Actions on the target such as build and test should only be invoked manually
     * and explicitly. For example, triggering a build on all targets in the workspace
     * should by default not include this target.
     *
     * The original motivation to add the "manual" tag comes from a similar functionality
     * that exists in Bazel, where targets with this tag have to be specified explicitly
     * on the command line. */
    Manual,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
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

/* Included in notifications of tasks or requests to signal the completion state. */
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum StatusCode {
    /** Execution was successful. */
    Ok = 1,
    /** Execution failed. */
    #[default]
    Error = 2,
    /** Execution was cancelled. */
    Cancelled = 3,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::test_serialization;

    use super::*;

    #[test]
    fn text_document_identifier() {
        test_serialization(&TextDocumentIdentifier::default(), r#"{"uri":""}"#);
    }

    #[test]
    fn build_client_capabilities() {
        test_serialization(
            &BuildClientCapabilities {
                language_ids: vec!["test_languageId".to_string()],
            },
            r#"{"languageIds":["test_languageId"]}"#,
        );
        test_serialization(
            &BuildClientCapabilities {
                language_ids: vec![],
            },
            r#"{"languageIds":[]}"#,
        );
    }

    #[test]
    fn build_server_capabilities() {
        let test_data = BuildServerCapabilities {
            compile_provider: Some(CompileProvider::default()),
            test_provider: Some(TestProvider::default()),
            run_provider: Some(RunProvider::default()),
            debug_provider: Some(DebugProvider::default()),
            inverse_sources_provider: Some(true),
            dependency_sources_provider: Some(true),
            dependency_modules_provider: Some(true),
            resources_provider: Some(true),
            output_paths_provider: Some(true),
            build_target_changed_provider: Some(true),
            jvm_run_environment_provider: Some(true),
            jvm_test_environment_provider: Some(true),
            can_reload: Some(true),
        };

        test_serialization(
            &test_data,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );

        let mut modified = test_data.clone();
        modified.compile_provider = None;
        test_serialization(
            &modified,
            r#"{"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.test_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.run_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.debug_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.inverse_sources_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.dependency_sources_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.dependency_modules_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.resources_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.output_paths_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.build_target_changed_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.jvm_run_environment_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmTestEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data.clone();
        modified.jvm_test_environment_provider = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"canReload":true}"#,
        );
        modified = test_data;
        modified.can_reload = None;
        test_serialization(
            &modified,
            r#"{"compileProvider":{"languageIds":[]},"testProvider":{"languageIds":[]},"runProvider":{"languageIds":[]},"debugProvider":{"languageIds":[]},"inverseSourcesProvider":true,"dependencySourcesProvider":true,"dependencyModulesProvider":true,"resourcesProvider":true,"outputPathsProvider":true,"buildTargetChangedProvider":true,"jvmRunEnvironmentProvider":true,"jvmTestEnvironmentProvider":true}"#,
        );
    }

    #[test]
    fn compile_provider() {
        test_serialization(
            &CompileProvider {
                language_ids: vec!["test_languageId".to_string()],
            },
            r#"{"languageIds":["test_languageId"]}"#,
        );
        test_serialization(
            &CompileProvider {
                language_ids: vec![],
            },
            r#"{"languageIds":[]}"#,
        );
    }

    #[test]
    fn run_provider() {
        test_serialization(
            &RunProvider {
                language_ids: vec!["test_languageId".to_string()],
            },
            r#"{"languageIds":["test_languageId"]}"#,
        );
        test_serialization(
            &RunProvider {
                language_ids: vec![],
            },
            r#"{"languageIds":[]}"#,
        );
    }

    #[test]
    fn debug_provider() {
        test_serialization(
            &DebugProvider {
                language_ids: vec!["test_languageId".to_string()],
            },
            r#"{"languageIds":["test_languageId"]}"#,
        );
        test_serialization(
            &DebugProvider {
                language_ids: vec![],
            },
            r#"{"languageIds":[]}"#,
        );
    }

    #[test]
    fn test_provider() {
        test_serialization(
            &TestProvider {
                language_ids: vec!["test_languageId".to_string()],
            },
            r#"{"languageIds":["test_languageId"]}"#,
        );
        test_serialization(
            &TestProvider {
                language_ids: vec![],
            },
            r#"{"languageIds":[]}"#,
        );
    }

    #[test]
    fn build_target() {
        let test_data = BuildTarget {
            id: BuildTargetIdentifier::default(),
            display_name: Some("test_displayName".to_string()),
            base_directory: Some("test_baseDirectory".to_string()),
            tags: vec![BuildTargetTag::Test],
            capabilities: BuildTargetCapabilities::default(),
            language_ids: vec!["test_languageId".to_string()],
            dependencies: vec![BuildTargetIdentifier::default()],
            data_kind: Some("test_dataKind".to_string()),
            data: Some(RustBuildTarget::default()),
        };

        test_serialization(
            &test_data,
            r#"{"id":{"uri":""},"displayName":"test_displayName","baseDirectory":"test_baseDirectory","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[{"uri":""}],"dataKind":"test_dataKind","data":{"edition":"2015","requiredFeatures":[]}}"#,
        );

        let mut modified = test_data.clone();
        modified.display_name = None;
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"baseDirectory":"test_baseDirectory","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[{"uri":""}],"dataKind":"test_dataKind","data":{"edition":"2015","requiredFeatures":[]}}"#,
        );
        modified = test_data.clone();
        modified.base_directory = None;
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"displayName":"test_displayName","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[{"uri":""}],"dataKind":"test_dataKind","data":{"edition":"2015","requiredFeatures":[]}}"#,
        );
        modified = test_data.clone();
        modified.tags = vec![];
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"displayName":"test_displayName","baseDirectory":"test_baseDirectory","tags":[],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[{"uri":""}],"dataKind":"test_dataKind","data":{"edition":"2015","requiredFeatures":[]}}"#,
        );
        modified = test_data.clone();
        modified.language_ids = vec![];
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"displayName":"test_displayName","baseDirectory":"test_baseDirectory","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":[],"dependencies":[{"uri":""}],"dataKind":"test_dataKind","data":{"edition":"2015","requiredFeatures":[]}}"#,
        );
        modified = test_data.clone();
        modified.dependencies = vec![];
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"displayName":"test_displayName","baseDirectory":"test_baseDirectory","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[],"dataKind":"test_dataKind","data":{"edition":"2015","requiredFeatures":[]}}"#,
        );
        modified = test_data;
        modified.data_kind = None;
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"displayName":"test_displayName","baseDirectory":"test_baseDirectory","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[{"uri":""}],"data":{"edition":"2015","requiredFeatures":[]}}"#,
        );
        modified.data = None;
        test_serialization(
            &modified,
            r#"{"id":{"uri":""},"displayName":"test_displayName","baseDirectory":"test_baseDirectory","tags":["test"],"capabilities":{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false},"languageIds":["test_languageId"],"dependencies":[{"uri":""}]}"#,
        );
    }

    #[test]
    fn rust_build_target() {
        test_serialization(
            &RustBuildTarget {
                edition: Edition::default(),
                required_features: vec!["test_requiredFeature".to_string()],
            },
            r#"{"edition":"2015","requiredFeatures":["test_requiredFeature"]}"#,
        );
        test_serialization(
            &RustBuildTarget {
                edition: Edition::default(),
                required_features: vec![],
            },
            r#"{"edition":"2015","requiredFeatures":[]}"#,
        );
    }

    #[test]
    fn build_target_identifier() {
        test_serialization(&BuildTargetIdentifier::default(), r#"{"uri":""}"#);
    }

    #[test]
    fn build_target_tag() {
        test_serialization(&BuildTargetTag::Library, r#""library""#);
        test_serialization(&BuildTargetTag::Application, r#""application""#);
        test_serialization(&BuildTargetTag::Test, r#""test""#);
        test_serialization(&BuildTargetTag::IntegrationTest, r#""integration-test""#);
        test_serialization(&BuildTargetTag::Benchmark, r#""benchmark""#);
        test_serialization(&BuildTargetTag::NoIde, r#""no-ide""#);
        test_serialization(&BuildTargetTag::Manual, r#""manual""#);
    }

    #[test]
    fn build_target_capabilities() {
        test_serialization(
            &BuildTargetCapabilities::default(),
            r#"{"canCompile":false,"canTest":false,"canRun":false,"canDebug":false}"#,
        );
    }

    #[test]
    fn status_code() {
        test_serialization(&StatusCode::Ok, r#"1"#);
        test_serialization(&StatusCode::Error, r#"2"#);
        test_serialization(&StatusCode::Cancelled, r#"3"#);
    }
}
