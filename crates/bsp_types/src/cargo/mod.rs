mod cargo_build_server;
mod cargo_build_target;
mod cargo_features_state_result;
mod package_features;
mod set_cargo_features_params;
mod set_cargo_features_result;

pub use cargo_build_server::*;
pub use cargo_build_target::*;
pub use cargo_features_state_result::*;
pub use package_features::*;
pub use set_cargo_features_params::*;
pub use set_cargo_features_result::*;

#[cfg(test)]
pub mod tests {
    use crate::bsp::BuildTargetIdentifier;
    use crate::rust::FeatureDependencyGraph;
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;

    pub const PACKAGE_ID: &str = "package_id";
    pub const PACKAGE_ID2: &str = "package_id2";
    pub const FEATURE: &str = "feature";
    pub const FEATURE2: &str = "feature2";
    pub const TARGET_ID: &str = "target";
    pub const TARGET_ID2: &str = "target2";

    pub fn example_package_features(pid: &str, f1: &str) -> PackageFeatures {
        PackageFeatures {
            package_id: pid.into(),
            enabled_features: vec![f1.into()].into_iter().collect(),
            available_features: FeatureDependencyGraph::new(BTreeMap::from([(
                f1.into(),
                BTreeSet::new(),
            )])),
            targets: vec![
                BuildTargetIdentifier {
                    uri: TARGET_ID.into(),
                },
                BuildTargetIdentifier {
                    uri: TARGET_ID2.into(),
                },
            ],
        }
    }
}
