//! [`ProjectModel`] obtains and stores information about the Rust project.

use bsp_types::basic_bsp_structures::Edition as BspEdition;
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

pub(crate) fn metadata_edition_to_bsp_edition(metadata_edition: Edition) -> BspEdition {
    BspEdition::new(metadata_edition.as_str())
}
