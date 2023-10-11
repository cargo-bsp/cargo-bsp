use serde::{Deserialize, Serialize};

use crate::*;
use std::collections::BTreeMap;

/// The RustDependencies is a mapping between
/// package id and the package's dependencies info.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustDependencies(pub BTreeMap<String, Vec<RustDependency>>);

impl RustDependencies {
    pub fn new(input: BTreeMap<String, Vec<RustDependency>>) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for RustDependencies {
    type Target = BTreeMap<String, Vec<RustDependency>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
