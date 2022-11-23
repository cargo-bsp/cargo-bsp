use std::fmt::Debug;

use jsonrpsee_core::Error;
use jsonrpsee_core::traits::ToRpcParams;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

pub struct RequestWrapper<T>
where
    T: Serialize + MethodName,
{
    pub request_params: T,
}
impl<T> ToRpcParams for RequestWrapper<T>
where
    T: Serialize + MethodName,
{
    fn to_rpc_params(self) -> Result<Option<Box<RawValue>>, Error> {
        serde_json::to_string(&self.request_params)
            .map(|x| RawValue::from_string(x).ok())
            .map_err(Into::into)
    }
}

pub trait MethodName {
    fn get_method_name() -> &'static str;
}

/**  A resource identifier that is a valid URI according
* to rfc3986: * https://tools.ietf.org/html/rfc3986 */
type Uri = String;

/** Client's initializing request */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildParams<T = ()> {
    /** Name of the client */
    pub display_name: String,

    /** The version of the client */
    pub version: String,

    /** The BSP version that the client speaks */
    pub bsp_version: String,

    /** The rootUri of the workspace */
    pub root_uri: Uri,

    /** The capabilities of the client */
    pub capabilities: BuildClientCapabilities,

    /** Additional metadata about the client */
    pub data: Option<T>,
}

impl MethodName for InitializeBuildParams {
    fn get_method_name() -> &'static str {
        "build/initialize"
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildClientCapabilities {
    /** The languages that this client supports.
     * The ID strings for each language is defined in the LSP.
     * The server must never respond with build targets for other
     * languages than those that appear in this list. */
    pub language_ids: Vec<String>,
}

/** Server's response for client's InitializeBuildParams request */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializeBuildResult<T = ()> {
    /** Name of the server */
    pub display_name: String,

    /** The version of the server */
    pub version: String,

    /** The BSP version that the server speaks */
    pub bsp_version: String,

    /** The capabilities of the build server */
    pub capabilities: BuildServerCapabilities,

    /** Additional metadata about the server */
    pub data: Option<T>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BuildServerCapabilities {
    /** The languages the server supports compilation via method buildTarget/compile. */
    pub compile_provider: Option<CompileProvider>,

    /** The languages the server supports test execution via method buildTarget/test */
    pub test_provider: Option<TestProvider>,

    /** The languages the server supports run via method buildTarget/run */
    pub run_provider: Option<RunProvider>,

    /** The languages the server supports debugging via method debugSession/start */
    pub debug_provider: Option<DebugProvider>,

    /** The server can provide a list of targets that contain a
     * single text document via the method buildTarget/inverseSources */
    pub inverse_sources_provider: Option<bool>,

    /** The server provides sources for library dependencies
     * via method buildTarget/dependencySources */
    pub dependency_sources_provider: Option<bool>,

    /** The server can provide a list of dependency modules (libraries with meta information)
     * via method buildTarget/dependencyModules */
    pub dependency_modules_provider: Option<bool>,

    /** The server provides all the resource dependencies
     * via method buildTarget/resources */
    pub resources_provider: Option<bool>,

    /** The server provides all output paths
     * via method buildTarget/outputPaths */
    pub output_paths_provider: Option<bool>,

    /** The server sends notifications to the client on build
     * target change events via buildTarget/didChange */
    pub build_target_changed_provider: Option<bool>,

    /** The server can respond to `buildTarget/jvmRunEnvironment` requests with the
     * necessary information required to launch a Java process to run a main class. */
    pub jvm_run_environment_provider: Option<bool>,

    /** The server can respond to `buildTarget/jvmTestEnvironment` requests with the
     * necessary information required to launch a Java process for testing or
     * debugging. */
    pub jvm_test_environment_provider: Option<bool>,

    /** Reloading the build state through workspace/reload is supported */
    pub can_reload: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompileProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RunProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DebugProvider {
    pub language_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TestProvider {
    pub language_ids: Vec<String>,
}
