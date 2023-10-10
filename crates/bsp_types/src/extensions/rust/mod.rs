mod rust_workspace;

pub use rust_workspace::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Feature(pub String);

impl Feature {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for Feature {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl FeatureDependencyGraph {
    pub fn new(input: BTreeMap<Feature, BTreeSet<Feature>>) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for FeatureDependencyGraph {
    type Target = BTreeMap<Feature, BTreeSet<Feature>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// The Rust edition.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustEdition(pub std::borrow::Cow<'static, str>);

impl RustEdition {
    pub const E2015: RustEdition = RustEdition::new("2015");
    pub const E2018: RustEdition = RustEdition::new("2018");
    pub const E2021: RustEdition = RustEdition::new("2021");

    pub const fn new(tag: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(tag))
    }
}
