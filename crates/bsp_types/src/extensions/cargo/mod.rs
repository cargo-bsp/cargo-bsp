//! [`CargoExtension`] provides implementation of the BSP structures for Cargo extension
//! (not yet implemented in BSP).

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub use cargo_build_target::*;
pub use cargo_features_state::*;
pub use set_cargo_features_state::*;

mod cargo_build_target;
mod cargo_features_state;
mod set_cargo_features_state;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Feature(pub String);

impl std::ops::Deref for Feature {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for Feature {
    fn from(input: String) -> Self {
        Self(input)
    }
}

impl From<&str> for Feature {
    fn from(input: &str) -> Self {
        Self(input.to_string())
    }
}

/// The feature dependency graph is a mapping between
/// feature and the features it turns on
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeatureDependencyGraph(pub BTreeMap<Feature, BTreeSet<Feature>>);

impl std::ops::Deref for FeatureDependencyGraph {
    type Target = BTreeMap<Feature, BTreeSet<Feature>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<BTreeMap<Feature, BTreeSet<Feature>>> for FeatureDependencyGraph {
    fn from(input: BTreeMap<Feature, BTreeSet<Feature>>) -> Self {
        Self(input)
    }
}
