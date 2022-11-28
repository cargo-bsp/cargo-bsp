use jsonrpsee_core::traits::ToRpcParams;
use jsonrpsee_core::Error;
use serde::Serialize;
use serde_json::value::RawValue;

/*
//dev: decide what to do wit shutdown_build_request and reload_request

//dev: decide whether we want to expose initialize and similar modules
mod initialize; //example: cargo_bsp::bsp_types::InitializeParams::new()
pub use initialize::*;
//---------------------
pub mod initialize //example: cargo bsp::bsp_types::initialize::InitializeParams::new()
*/

mod basic_bsp_structures;
pub use basic_bsp_structures::*;

mod initialize;
pub use initialize::*;

mod workspace_build_targets;
pub use workspace_build_targets::*;

mod build_target_sources;
pub use build_target_sources::*;

mod shutdown_build_request;
mod reload_request;

mod inverse_sources_request;
pub use inverse_sources_request::*;

mod dependency_sources_request;
pub use dependency_sources_request::*;

mod resources_request;
pub use resources_request::*;

mod resource_request;
pub use resource_request::*;

mod output_paths_request;
pub use output_paths_request::*;

mod compile_request;
pub use compile_request::*;

mod test_request;
pub use test_request::*;

mod run_request;
pub use run_request::*;

mod debug_request;
pub use debug_request::*;

mod clean_cache_request;
pub use clean_cache_request::*;

// Trait for all the request types and maybe in future notification types
pub trait MethodName {
    fn get_method_name() -> &'static str;
}

// Simple Wrapper for client
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
