use serde::{Deserialize, Serialize};

use crate::*;
use std::collections::BTreeMap;

/// The RustRawDependencies is a mapping between
/// package id and the package's raw dependencies info.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustRawDependencies(pub BTreeMap<String, Vec<RustRawDependency>>);

impl RustRawDependencies {
    pub fn new(input: BTreeMap<String, Vec<RustRawDependency>>) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for RustRawDependencies {
    type Target = BTreeMap<String, Vec<RustRawDependency>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
