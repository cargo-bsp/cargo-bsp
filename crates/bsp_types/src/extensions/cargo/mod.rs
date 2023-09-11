//! [`CargoExtension`] provides implementation of the BSP structures for Cargo extension
//! (not yet implemented in BSP).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use cargo_features_state::*;
pub use set_cargo_features_state::*;

mod cargo_features_state;
mod set_cargo_features_state;
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Feature(pub String);

/// Hashmap where key is a feature name and the value are names of other features it enables.
/// Includes pair for default features if default is defined
pub type FeaturesDependencyGraph = BTreeMap<Feature, Vec<Feature>>;

impl From<&str> for Feature {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
