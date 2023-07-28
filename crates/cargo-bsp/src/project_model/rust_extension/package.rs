//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are partially responsible
//! for preparing the data for RustWorkspaceRequest response.

use crate::project_model::rust_extension::{
    metadata_edition_to_rust_extension_edition, target::metadata_targets_to_rust_extension_targets,
};
use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::{RustFeature, RustPackage, RustPackageOrigin, RustRawDependency};
use bsp_types::BuildTargetIdentifier;
use cargo_metadata::{DependencyKind, Package};
use std::collections::HashMap;

fn dependency_kind_to_string(dependency: DependencyKind) -> Option<String> {
    match dependency {
        DependencyKind::Build => Some("build".to_string()),
        DependencyKind::Development => Some("dev".to_string()),
        DependencyKind::Normal => None, // Cargo metadata output defaults to Null, when dependency is normal
        _ => None,
    }
}

fn resolve_origin(mut package: RustPackage, workspace: &ProjectWorkspace) -> RustPackage {
    // todo check if it is a workspace package or external lib
    // todo check if it is a stdlib ord stdlib dep in InteliJ rust
    if workspace.is_package_part_of_workspace(&package.id) {
        package.origin = RustPackageOrigin::Workspace;
    } else {
        package.origin = RustPackageOrigin::Dependency;
    }
    package
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

fn metadata_package_to_rust_extension_package(metadata_package: Package) -> RustPackage {
    let all_targets = metadata_targets_to_rust_extension_targets(metadata_package.targets);
    RustPackage {
        id: metadata_package.id.clone().to_string(),
        version: metadata_package.version.to_string(),
        edition: metadata_edition_to_rust_extension_edition(metadata_package.edition),
        origin: RustPackageOrigin::Unset, // This field will be resolved later
        source: metadata_package.source.map(|s| s.to_string()),
        features: metadata_features_to_rust_extension_features(metadata_package.features),
        enabled_features: Default::default(), // todo resolve from Cargo metadata -> resolved -> nodes (grouped by packageId) -> features.
        // In our case targets = all_targets. This field is needed for Bazel //TODO (Check)
        targets: all_targets.clone(),
        all_targets,
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
            let rust_package = metadata_package_to_rust_extension_package(package);
            resolve_origin(rust_package, workspace)
        })
        .collect()
}

pub fn resolve_raw_dependencies(
    workspace: &ProjectWorkspace,
    targets: &[BuildTargetIdentifier],
) -> HashMap<String, RustRawDependency> {
    targets
        .iter()
        .filter_map(|t| workspace.get_package_related_to_target(t))
        .flat_map(|p| {
            p.dependencies
                .iter()
                .cloned()
                .map(|d| {
                    let rust_raw_dep = RustRawDependency {
                        name: d.name,
                        optional: d.optional,
                        uses_default_features: d.uses_default_features,
                        features: d.features.into_iter().map(|f| f.0).collect(),
                        rename: d.rename,
                        kind: dependency_kind_to_string(d.kind),
                        target: d.target.map(|p| p.to_string()),
                    };
                    (p.id.clone(), rust_raw_dep)
                })
                .collect::<Vec<(String, RustRawDependency)>>()
        })
        .collect()
}
