//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are partially responsible
//! for preparing the data for RustWorkspaceRequest response.

use crate::project_model::cargo_package::CargoPackage;
use crate::project_model::metadata_edition_to_bsp_edition;
use crate::project_model::rust_extension::{
    find_node, get_nodes_from_metadata, target::metadata_targets_to_rust_extension_targets,
};
use crate::project_model::workspace::ProjectWorkspace;
use crate::utils::uri::file_uri;
use bsp_types::extensions::{Feature, FeatureDependencyGraph, RustPackage, RustPackageOrigin};
use bsp_types::BuildTargetIdentifier;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};

fn resolve_origin(package: &mut RustPackage, workspace: &ProjectWorkspace) {
    if workspace.is_package_part_of_workspace(&package.id) {
        package.origin = RustPackageOrigin::WORKSPACE;
    } else {
        package.origin = RustPackageOrigin::DEPENDENCY;
    }
}

fn set_and_resolve_enabled_features(
    workspace: &mut ProjectWorkspace,
    package: &mut RustPackage,
    nodes: &[cargo_metadata::Node],
) {
    if let Some(n) = find_node(
        nodes,
        &package.id,
        "Proceeding with empty enabled features.",
    ) {
        package.enabled_features = n
            .features
            .iter()
            .map(|f| Feature::from(f.as_str()))
            .collect();
        // Set enabled features in server's state.
        let features = n
            .features
            .clone()
            .into_iter()
            .map(Feature)
            .collect::<BTreeSet<Feature>>();
        workspace.set_features_for_the_package(package.id.clone(), &features);
    }
}

fn metadata_features_to_rust_extension_features(
    metadata_features: BTreeMap<String, Vec<String>>,
) -> FeatureDependencyGraph {
    metadata_features
        .into_iter()
        .map(|(f, deps)| (Feature(f), deps.into_iter().map(Feature).collect()))
        .collect::<BTreeMap<Feature, BTreeSet<Feature>>>()
        .into()
}

fn metadata_package_to_rust_extension_package(
    metadata_package: cargo_metadata::Package,
) -> RustPackage {
    let all_targets = metadata_targets_to_rust_extension_targets(metadata_package.targets);
    RustPackage {
        id: metadata_package.id.clone().to_string(),
        root_url: file_uri(metadata_package.manifest_path.parent().unwrap().to_string()),
        name: metadata_package.name.clone(),
        version: metadata_package.version.to_string(),
        edition: metadata_edition_to_bsp_edition(metadata_package.edition),
        source: metadata_package.source.map(|s| s.to_string()),
        features: metadata_features_to_rust_extension_features(metadata_package.features),
        // In our case targets = all_targets. This field is needed for Bazel
        resolved_targets: all_targets.clone(),
        all_targets,
        // The rest of the fields is resolved later
        ..RustPackage::default()
    }
}

/// Returns a list of rust extension packages from which provided targets depend on
pub fn get_rust_packages_related_to_targets(
    workspace: &mut ProjectWorkspace,
    metadata: &cargo_metadata::Metadata,
    targets: &[BuildTargetIdentifier],
) -> Vec<RustPackage> {
    let target_related_packages_names: Vec<String> = targets
        .iter()
        .filter_map(|t| workspace.get_package_related_to_target(t))
        .flat_map(|p| find_all_packages(p, &metadata.packages))
        .collect::<HashSet<_>>()
        .into_iter()
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
            if workspace.is_package_part_of_workspace(rust_package.id.as_str()) {
                set_and_resolve_enabled_features(workspace, &mut rust_package, &nodes);
            }
            rust_package
        })
        .collect()
}

fn find_all_packages(package: &CargoPackage, packages: &[cargo_metadata::Package]) -> Vec<String> {
    if let Some(package) = packages.iter().find(|p| p.name == package.name) {
        let mut next_dependencies: VecDeque<&cargo_metadata::Package> = VecDeque::from([package]);
        let mut checked_dependencies: HashSet<String> = HashSet::from([package.name.clone()]);
        let mut all_package_names: Vec<String> = vec![package.name.clone()];

        while let Some(next) = next_dependencies.pop_front() {
            for dependency in &next.dependencies {
                if !checked_dependencies.contains(&dependency.name) {
                    checked_dependencies.insert(dependency.name.clone());
                    if let Some(p) = packages.iter().find(|p| p.name == dependency.name) {
                        all_package_names.push(p.name.clone());
                        next_dependencies.push_back(p);
                    }
                }
            }
        }
        return all_package_names;
    }
    Vec::new()
}
