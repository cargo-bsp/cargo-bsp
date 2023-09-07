//! [`ProjectModel`] obtains and stores information about the Rust project.

use bsp_types::extensions::Feature;

mod _unit_tests_discovery;
pub(crate) mod build_target_mappings;
pub(crate) mod cargo_package;
pub(crate) mod package_dependency;
pub(crate) mod project_manifest;
pub(crate) mod sources;
pub(crate) mod target_details;
pub mod workspace;

pub trait DefaultFeature {
    fn default_feature_name() -> Self;
}
impl DefaultFeature for Feature {
    fn default_feature_name() -> Feature {
        Feature::from("default")
    }
}
