//! [`CargoCommunication`] manages the communication with Cargo, such as
//! preparing the appropriate commands, executing and parsing information from them,
//! and preparing the appropriate responses for the client based on the given information.

pub(crate) mod cargo_actor;
mod cargo_handle;
pub(crate) mod cargo_types;
mod check;
pub(crate) mod execution;
pub(crate) mod request_handle;
mod utils;
