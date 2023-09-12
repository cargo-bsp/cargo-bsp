//! [`Execution`] handles communication with Cargo regarding Compile/Run/Test Requests.

mod cargo_messages_handle;
pub(crate) mod cargo_types;
pub(crate) mod execution_actor;
mod execution_actor_sender;
mod execution_actor_state;
mod execution_actor_unit_graph;
mod execution_handle;
mod utils;
