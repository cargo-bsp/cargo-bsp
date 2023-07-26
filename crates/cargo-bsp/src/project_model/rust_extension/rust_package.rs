//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are partially responsible
//! for preparing the data for RustWorkspaceRequest response.

use crate::project_model::rust_extension::{
    metadata_edition_to_rust_extension_edition,
    rust_target::metadata_targets_to_rust_extension_targets,
};
use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::RustPackage;
use bsp_types::BuildTargetIdentifier;

fn metadata_package_to_rust_extension_package(
    metadata_package: cargo_metadata::Package,
) -> RustPackage {
    RustPackage {
        id: metadata_package.id.clone().to_string(),
        version: metadata_package.version.to_string(),
        edition: metadata_edition_to_rust_extension_edition(metadata_package.edition),
        source: metadata_package.source.map(|s| s.to_string()),
        enabled_features: metadata_package.features.keys().cloned().collect(),
        targets: metadata_targets_to_rust_extension_targets(metadata_package.targets),
        origin: Default::default(),      //todo wait for enum
        all_targets: Default::default(), //todo find out what is it
        features: Default::default(),

        cfg_options: Default::default(),
        env: Default::default(),
        out_dir_url: Default::default(),
        proc_macro_artifact: Default::default(),
    }
}

/// Returns a list of rust extension packages from which provided targets depend on
pub fn get_rust_packages_related_to_targets(
    workspace: &ProjectWorkspace,
    targets: &[BuildTargetIdentifier],
) -> Vec<RustPackage> {
    let target_related_packages_names: Vec<String> = targets
        .iter()
        .filter_map(|t| workspace.get_package_related_to_target(t))
        .flat_map(|p| {
            let mut names: Vec<String> = p.dependencies.iter().map(|d| d.name.clone()).collect();
            names.push(p.name.clone());
            names
        })
        .collect();

    target_related_packages_names
        .iter()
        .map(|n| {
            let package = workspace
                .all_packages
                .iter()
                .find(|p| p.name == *n)
                .unwrap()
                .clone();
            metadata_package_to_rust_extension_package(package)
        })
        .collect()
}
