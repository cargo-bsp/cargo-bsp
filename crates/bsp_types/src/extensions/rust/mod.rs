mod rust_workspace;

pub use rust_workspace::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
