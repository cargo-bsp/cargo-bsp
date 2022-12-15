use jsonrpsee_core::traits::ToRpcParams;
use jsonrpsee_core::Error;
use serde::Serialize;
use serde_json::value::RawValue;

//dev: decide what to do wit shutdown_build_request and reload_request

mod basic_bsp_structures;
pub use basic_bsp_structures::*;

mod workspace_build_targets;
pub use workspace_build_targets::*;

pub mod requests;

pub mod notifications;

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
