//! [`CargoExtension`] provides implementation of the BSP structures for Cargo extension
//! (not yet implemented in BSP).

pub use cargo_build_target::*;
pub use cargo_features_state::*;
pub use set_cargo_features_state::*;

mod cargo_build_target;
mod cargo_features_state;
mod set_cargo_features_state;
