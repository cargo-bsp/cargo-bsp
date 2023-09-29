//! [`CargoTypes`] provides necessary structures for handling information
//! from Cargo build/run/test commands.

pub(crate) mod cargo_result;
pub(crate) mod create_unit_graph_command;
pub(crate) mod origin_id;
pub(super) mod publish_diagnostics;
pub(super) mod test;
pub(super) mod unit_graph;
