use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustCfgOptions(pub BTreeMap<String, Vec<String>>);

impl RustCfgOptions {
    pub fn new(input: BTreeMap<String, Vec<String>>) -> Self {
        Self(input)
    }
}

impl std::ops::Deref for RustCfgOptions {
    type Target = BTreeMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
