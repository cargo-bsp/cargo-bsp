//! [`CargoExtension`] provides implementation of the BSP structures for Cargo extension
//! (not yet implemented in BSP).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use cargo_features_state::*;
pub use disable_cargo_features::*;
pub use enable_cargo_features::*;

mod cargo_features_state;
mod disable_cargo_features;
mod enable_cargo_features;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Feature(pub String);

pub type FeaturesDependencyGraph = BTreeMap<Feature, Vec<Feature>>;

impl From<&str> for Feature {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
