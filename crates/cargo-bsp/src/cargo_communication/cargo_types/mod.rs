//! [`CargoTypes`] provides all necessary structures for handling information
//! from Cargo commands.

pub(crate) mod cargo_command;
pub(crate) mod cargo_result;
pub(super) mod event;
pub(crate) mod params_target;
pub(super) mod publish_diagnostics;
pub(super) mod test;
pub(super) mod unit_graph;
