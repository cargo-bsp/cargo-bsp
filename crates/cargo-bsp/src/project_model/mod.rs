//! [`ProjectModel`] obtains and stores information about the Rust project.

use bsp_types::extensions::{Feature, FeatureDependencyGraph, RustEdition};
use cargo_metadata::{Edition, Package};
use std::collections::{BTreeMap, BTreeSet};

mod _unit_tests_discovery;
pub(crate) mod build_target_mappings;
pub(crate) mod cargo_package;
pub(crate) mod package_dependency;
pub(crate) mod project_manifest;
pub(crate) mod rust_extension;
pub(crate) mod sources;
pub(crate) mod target_details;
pub mod workspace;

pub const RUST_ID: &str = "rust";

pub(crate) fn metadata_edition_to_bsp_edition(metadata_edition: Edition) -> RustEdition {
    RustEdition::new(metadata_edition.as_str())
}

pub trait DefaultFeature {
    fn default_feature_name() -> Self;
}
impl DefaultFeature for Feature {
    fn default_feature_name() -> Feature {
        Feature::from("default")
    }
}

pub trait CreateFeatureDependencyGraph {
    fn create_features_dependency_graph(metadata_package: &Package) -> Self;
}

impl CreateFeatureDependencyGraph for FeatureDependencyGraph {
    fn create_features_dependency_graph(metadata_package: &Package) -> Self {
        Self::new(
            metadata_package
                .features
                .clone()
                .into_iter()
                .map(|(f, df)| (Feature(f), df.into_iter().map(Feature).collect()))
                .collect::<BTreeMap<Feature, BTreeSet<Feature>>>(),
        )
    }
}
