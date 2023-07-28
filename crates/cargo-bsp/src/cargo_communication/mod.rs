//! [`CargoCommunication`] manages the communication with Cargo, such as
//! preparing the appropriate commands, executing and parsing information from them,
//! and preparing the appropriate responses for the client based on the given information.

pub(crate) mod cargo_actor;
mod cargo_check;
mod cargo_handle;
mod cargo_messages_handle;
pub(crate) mod cargo_types;
pub(crate) mod request_actor;
mod request_actor_sender;
mod request_actor_state;
mod request_actor_unit_graph;
pub(crate) mod request_handle;
mod utils;
