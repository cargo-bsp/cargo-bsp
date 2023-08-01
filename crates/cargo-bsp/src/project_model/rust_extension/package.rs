//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are partially responsible
//! for preparing the data for RustWorkspaceRequest response.

use crate::project_model::rust_extension::{
    find_node, get_nodes_from_metadata, metadata_edition_to_rust_extension_edition,
    target::metadata_targets_to_rust_extension_targets,
};
use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::{RustFeature, RustPackage, RustPackageOrigin};
use bsp_types::BuildTargetIdentifier;
use std::collections::HashMap;

fn resolve_origin(package: &mut RustPackage, workspace: &ProjectWorkspace) {
    // todo check if it is a stdlib ord stdlib dep in InteliJ rust
    if workspace.is_package_part_of_workspace(&package.id) {
        package.origin = RustPackageOrigin::Workspace;
    } else {
        package.origin = RustPackageOrigin::Dependency;
    }
}

fn resolve_enabled_features(package: &mut RustPackage, nodes: &[cargo_metadata::Node]) {
    if let Some(n) = find_node(
        nodes,
        &package.id,
        "Proceeding with empty enabled features.",
    ) {
        package.enabled_features = n.features.clone()
    }
}

fn metadata_features_to_rust_extension_features(
    metadata_features: HashMap<String, Vec<String>>,
) -> Vec<RustFeature> {
    metadata_features
        .into_iter()
        .map(|(f, deps)| RustFeature {
            name: f,
            dependencies: deps,
        })
        .collect()
}

fn metadata_package_to_rust_extension_package(
    metadata_package: cargo_metadata::Package,
) -> RustPackage {
    let all_targets = metadata_targets_to_rust_extension_targets(
        metadata_package.targets,
        &metadata_package.manifest_path,
    );
    RustPackage {
        id: metadata_package.id.clone().to_string(),
        version: metadata_package.version.to_string(),
        edition: metadata_edition_to_rust_extension_edition(metadata_package.edition),
        origin: RustPackageOrigin::Unset, // This field will be resolved later
        source: metadata_package.source.map(|s| s.to_string()),
        features: metadata_features_to_rust_extension_features(metadata_package.features),
        // In our case targets = all_targets. This field is needed for Bazel //TODO (Check)
        targets: all_targets.clone(),
        all_targets,
        // The rest of the fields is resolved later
        ..RustPackage::default()
    }
}

/// Returns a list of rust extension packages from which provided targets depend on
pub fn get_rust_packages_related_to_targets(
    workspace: &ProjectWorkspace,
    metadata: &cargo_metadata::Metadata,
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

    let nodes = get_nodes_from_metadata(metadata);

    target_related_packages_names
        .iter()
        .map(|n| {
            let package = metadata
                .packages
                .iter()
                .find(|p| p.name == *n)
                .unwrap()
                .clone();
            let mut rust_package = metadata_package_to_rust_extension_package(package);
            resolve_origin(&mut rust_package, workspace);
            resolve_enabled_features(&mut rust_package, &nodes);
            rust_package
        })
        .collect()
}
