use serde::{Deserialize, Serialize};

pub use disable_rust_features::*;
pub use enable_rust_features::*;
pub use rust_features_state::*;

mod disable_rust_features;
mod enable_rust_features;
mod rust_features_state;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Feature(pub String);

impl From<&str> for Feature {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
