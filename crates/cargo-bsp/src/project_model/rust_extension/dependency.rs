//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are used to resolve the dependencies part of the request.

use crate::project_model::package_dependency::PackageDependency;
use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::{RustDependency, RustRawDependency};
use bsp_types::BuildTargetIdentifier;
use cargo_metadata::DependencyKind;
use std::collections::HashMap;

fn dependency_kind_to_string(dependency: DependencyKind) -> Option<String> {
    match dependency {
        DependencyKind::Build => Some("build".to_string()),
        DependencyKind::Development => Some("dev".to_string()),
        DependencyKind::Normal => None, // Cargo metadata output defaults to Null, when dependency is normal
        _ => None,
    }
}

fn package_dependency_to_rust_dependency(
    package_dependency: PackageDependency,
) -> (String, RustDependency) {
    let source = package_dependency.source.unwrap();
    let rust_dep = RustDependency {
        name: package_dependency.name,
        target: package_dependency.target.map(|p| p.to_string()),
        dep_kinds: Default::default(), //todo find out what this field is for and how to obtain it
    };
    (source, rust_dep)
}

fn package_dependency_to_rust_raw_dependency(
    package_dependency: PackageDependency,
) -> RustRawDependency {
    RustRawDependency {
        name: package_dependency.name,
        optional: package_dependency.optional,
        uses_default_features: package_dependency.uses_default_features,
        features: package_dependency
            .features
            .into_iter()
            .map(|f| f.0)
            .collect(),
        rename: package_dependency.rename,
        kind: dependency_kind_to_string(package_dependency.kind),
        target: package_dependency.target.map(|p| p.to_string()),
    }
}

fn get_package_dependency_map(
    workspace: &ProjectWorkspace,
    targets: &[BuildTargetIdentifier],
) -> Vec<(String, PackageDependency)> {
    targets
        .iter()
        .filter_map(|t| workspace.get_package_related_to_target(t))
        .flat_map(|p| {
            p.dependencies
                .iter()
                .cloned()
                .map(|d| (p.id.clone(), d))
                .collect::<Vec<(String, PackageDependency)>>()
        })
        .collect()
}

pub fn resolve_dependencies(
    workspace: &ProjectWorkspace,
    targets: &[BuildTargetIdentifier],
) -> (
    HashMap<String, RustRawDependency>,
    HashMap<String, RustDependency>,
) {
    let dependencies: Vec<(String, PackageDependency)> =
        get_package_dependency_map(workspace, targets);

    let rust_dependencies: HashMap<String, RustDependency> = dependencies
        .iter()
        .map(|(_, d)| package_dependency_to_rust_dependency(d.clone()))
        .collect();

    let raw_dependencies: HashMap<String, RustRawDependency> = dependencies
        .into_iter()
        .map(|(package_id, d)| {
            (
                package_id.clone(),
                package_dependency_to_rust_raw_dependency(d),
            )
        })
        .collect();

    (raw_dependencies, rust_dependencies)
}
