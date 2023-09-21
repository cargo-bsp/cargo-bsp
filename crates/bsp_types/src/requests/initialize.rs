use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::{LanguageId, OtherData, URI};

/// Like the language server protocol, the initialize request is sent as the first request from the client to the server.
/// If the server receives a request or notification before the initialize request it should act as follows:
///
/// * For a request the response should be an error with code: -32002. The message can be picked by the server.
/// * Notifications should be dropped, except for the exit notification. This will allow the exit of a server without an initialize request.
///
/// Until the server has responded to the initialize request with an InitializeBuildResult, the client must not send any additional
/// requests or notifications to the server.
#[derive(Debug)]
pub enum BuildInitialize {}

impl Request for BuildInitialize {
    type Params = InitializeBuildParams;
    type Result = InitializeBuildResult;
    const METHOD: &'static str = "build/initialize";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildParams {
    /// Name of the client
    pub display_name: String,
    /// The version of the client
    pub version: String,
    /// The BSP version that the client speaks
    pub bsp_version: String,
    /// The rootUri of the workspace
    pub root_uri: URI,
    /// The capabilities of the client
    pub capabilities: BuildClientCapabilities,
    /// Additional metadata about the client
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<InitializeBuildParamsData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedInitializeBuildParamsData {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InitializeBuildParamsData {
    Named(NamedInitializeBuildParamsData),
    Other(OtherData),
}

impl InitializeBuildParamsData {}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildResult {
    /// Name of the server
    pub display_name: String,
    /// The version of the server
    pub version: String,
    /// The BSP version that the server speaks
    pub bsp_version: String,
    /// The capabilities of the build server
    pub capabilities: BuildServerCapabilities,
    /// Additional metadata about the server
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<InitializeBuildResultData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedInitializeBuildResultData {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InitializeBuildResultData {
    Named(NamedInitializeBuildResultData),
    Other(OtherData),
}

impl InitializeBuildResultData {}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildClientCapabilities {
    /// The languages that this client supports.
    /// The ID strings for each language is defined in the LSP.
    /// The server must never respond with build targets for other
    /// languages than those that appear in this list.
    pub language_ids: Vec<LanguageId>,
}

/// The capabilities of the build server.
/// Clients can use these capabilities to notify users what BSP endpoints can and
/// cannot be used and why.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerCapabilities {
    /// The languages the server supports compilation via method buildTarget/compile.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compile_provider: Option<CompileProvider>,
    /// The languages the server supports test execution via method buildTarget/test.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_provider: Option<TestProvider>,
    /// The languages the server supports run via method buildTarget/run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_provider: Option<RunProvider>,
    /// The languages the server supports debugging via method debugSession/start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_provider: Option<DebugProvider>,
    /// The server can provide a list of targets that contain a
    /// single text document via the method buildTarget/inverseSources
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse_sources_provider: Option<bool>,
    /// The server provides sources for library dependencies
    /// via method buildTarget/dependencySources
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_sources_provider: Option<bool>,
    /// The server can provide a list of dependency modules (libraries with meta information)
    /// via method buildTarget/dependencyModules
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dependency_modules_provider: Option<bool>,
    /// The server provides all the resource dependencies
    /// via method buildTarget/resources
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resources_provider: Option<bool>,
    /// The server provides all output paths
    /// via method buildTarget/outputPaths
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_paths_provider: Option<bool>,
    /// The server sends notifications to the client on build
    /// target change events via buildTarget/didChange
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_target_changed_provider: Option<bool>,
    /// The server can respond to `buildTarget/jvmRunEnvironment` requests with the
    /// necessary information required to launch a Java process to run a main class.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jvm_run_environment_provider: Option<bool>,
    /// The server can respond to `buildTarget/jvmTestEnvironment` requests with the
    /// necessary information required to launch a Java process for testing or
    /// debugging.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jvm_test_environment_provider: Option<bool>,
    /// The server can respond to `workspace/cargoFeaturesState` and
    /// `setCargoFeatures` requests. In other words, supports Cargo Features extension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cargo_features_provider: Option<bool>,
    /// Reloading the build state through workspace/reload is supported
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_reload: Option<bool>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileProvider {
    pub language_ids: Vec<LanguageId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunProvider {
    pub language_ids: Vec<LanguageId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugProvider {
    pub language_ids: Vec<LanguageId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestProvider {
    pub language_ids: Vec<LanguageId>,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn initialize_build_method() {
        assert_eq!(BuildInitialize::METHOD, "build/initialize");
    }

    #[test]
    fn initialize_build_params() {
        let test_data = InitializeBuildParams {
            display_name: "test_name".to_string(),
            version: "1.0.0".to_string(),
            bsp_version: "2.0.0".to_string(),
            root_uri: URI::from("file:///test"),
            capabilities: BuildClientCapabilities::default(),
            data: Some(InitializeBuildParamsData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        test_deserialization(
            r#"{"displayName":"test_name","version":"1.0.0","bspVersion":"2.0.0","rootUri":"file:///test","capabilities":{"languageIds":[]},"dataKind":"test_dataKind","data":{"dataKey":"dataValue"}}"#,
            &test_data,
        );

        test_deserialization(
            r#"{"displayName":"","version":"","bspVersion":"","rootUri":"","capabilities":{"languageIds":[]}}"#,
            &InitializeBuildParams::default(),
        );
    }

    #[test]
    fn initialize_build_result() {
        let test_data = InitializeBuildResult {
            display_name: "test_name".to_string(),
            version: "1.0.0".to_string(),
            bsp_version: "2.0.0".to_string(),
            capabilities: BuildServerCapabilities::default(),
            data: Some(InitializeBuildResultData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "displayName": "test_name",
          "version": "1.0.0",
          "bspVersion": "2.0.0",
          "capabilities": {},
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(InitializeBuildResult::default(),
            @r#"
        {
          "displayName": "",
          "version": "",
          "bspVersion": "",
          "capabilities": {}
        }
        "#
        );
    }

    #[test]
    fn build_client_capabilities() {
        let test_data = BuildClientCapabilities {
            language_ids: vec!["test_languageId".into()],
        };

        test_deserialization(r#"{"languageIds":["test_languageId"]}"#, &test_data);

        test_deserialization(r#"{"languageIds":[]}"#, &BuildClientCapabilities::default());
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
            cargo_features_provider: Some(true),
            can_reload: Some(true),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "compileProvider": {
            "languageIds": []
          },
          "testProvider": {
            "languageIds": []
          },
          "runProvider": {
            "languageIds": []
          },
          "debugProvider": {
            "languageIds": []
          },
          "inverseSourcesProvider": true,
          "dependencySourcesProvider": true,
          "dependencyModulesProvider": true,
          "resourcesProvider": true,
          "outputPathsProvider": true,
          "buildTargetChangedProvider": true,
          "jvmRunEnvironmentProvider": true,
          "jvmTestEnvironmentProvider": true,
          "cargoFeaturesProvider": true,
          "canReload": true
        }
        "#
        );
        assert_json_snapshot!(BuildServerCapabilities::default(),
            @"{}"
        );
    }

    #[test]
    fn compile_provider() {
        let test_data = CompileProvider {
            language_ids: vec!["test_languageId".into()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "languageIds": [
            "test_languageId"
          ]
        }
        "#
        );
        assert_json_snapshot!(CompileProvider::default(),
            @r#"
        {
          "languageIds": []
        }
        "#
        );
    }

    #[test]
    fn run_provider() {
        let test_data = RunProvider {
            language_ids: vec!["test_languageId".into()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "languageIds": [
            "test_languageId"
          ]
        }
        "#
        );
        assert_json_snapshot!(RunProvider::default(),
            @r#"
        {
          "languageIds": []
        }
        "#
        );
    }

    #[test]
    fn debug_provider() {
        let test_data = DebugProvider {
            language_ids: vec!["test_languageId".into()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "languageIds": [
            "test_languageId"
          ]
        }
        "#
        );
        assert_json_snapshot!(DebugProvider::default(),
            @r#"
        {
          "languageIds": []
        }
        "#
        );
    }

    #[test]
    fn test_provider() {
        let test_data = TestProvider {
            language_ids: vec!["test_languageId".into()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "languageIds": [
            "test_languageId"
          ]
        }
        "#
        );
        assert_json_snapshot!(TestProvider::default(),
            @r#"
        {
          "languageIds": []
        }
        "#
        );
    }
}
