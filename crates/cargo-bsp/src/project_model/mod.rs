//! [`ProjectModel`] obtains and stores information about the Rust project.

use bsp_types::extensions::Feature;
use bsp_types::basic_bsp_structures::RustEdition;
use cargo_metadata::Edition;

mod _unit_tests_discovery;
pub(crate) mod build_target_mappings;
pub(crate) mod cargo_package;
pub(crate) mod package_dependency;
pub(crate) mod project_manifest;
pub(crate) mod rust_extension;
pub(crate) mod sources;
pub(crate) mod target_details;
pub mod workspace;

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
