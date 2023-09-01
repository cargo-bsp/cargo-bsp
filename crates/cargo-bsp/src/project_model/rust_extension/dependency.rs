//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are used to resolve the dependencies part of the request.

use crate::project_model::rust_extension::{find_node, get_nodes_from_metadata};
use bsp_types::extensions::{
    PackageIdToRustDependency, PackageIdToRustRawDependency, RustDepKind, RustDepKindInfo,
    RustDependency, RustPackage, RustRawDependency,
};
use cargo_metadata::DependencyKind;

fn metadata_dependency_kind_to_string(metadata_dependency_kind: DependencyKind) -> Option<String> {
    match metadata_dependency_kind {
        DependencyKind::Build => Some("build".to_string()),
        DependencyKind::Development => Some("dev".to_string()),
        DependencyKind::Normal => None, // Cargo metadata output defaults to Null, when dependency is normal
        _ => None,
    }
}

fn package_dependency_to_rust_raw_dependency(
    package_dependency: cargo_metadata::Dependency,
) -> RustRawDependency {
    RustRawDependency {
        name: package_dependency.name,
        optional: package_dependency.optional,
        uses_default_features: package_dependency.uses_default_features,
        features: package_dependency.features.into_iter().collect(),
        rename: package_dependency.rename,
        kind: metadata_dependency_kind_to_string(package_dependency.kind),
        target: package_dependency.target.map(|p| p.to_string()),
    }
}

fn metadata_dep_kind_to_rust_dep_kind(metadata_dep_kind: DependencyKind) -> RustDepKind {
    match metadata_dep_kind {
        DependencyKind::Build => RustDepKind::Build,
        DependencyKind::Development => RustDepKind::Dev,
        DependencyKind::Normal => RustDepKind::Normal,
        DependencyKind::Unknown => RustDepKind::Unclassified,
    }
}

fn metadata_dep_kinds_info_to_rust_dep_kinds_info(
    metadata_dep_kinds_info: &[cargo_metadata::DepKindInfo],
) -> Vec<RustDepKindInfo> {
    metadata_dep_kinds_info
        .iter()
        .map(|d| RustDepKindInfo {
            kind: metadata_dep_kind_to_rust_dep_kind(d.kind),
            target: d.target.clone().map(|t| t.to_string()),
        })
        .collect()
}

fn metadata_node_dep_to_rust_dependency(node_dep: &cargo_metadata::NodeDep) -> RustDependency {
    RustDependency {
        name: Some(node_dep.name.clone()),
        pkg: node_dep.pkg.to_string(),
        dep_kinds: metadata_dep_kinds_info_to_rust_dep_kinds_info(&node_dep.dep_kinds),
    }
}

pub fn resolve_raw_dependencies(
    metadata: &cargo_metadata::Metadata,
    packages: &[RustPackage],
) -> PackageIdToRustRawDependency {
    packages
        .iter()
        .filter_map(|p| metadata.packages.iter().find(|wp| wp.id.repr == p.id))
        .map(|p| {
            let dependencies = p
                .dependencies
                .iter()
                .cloned()
                .map(package_dependency_to_rust_raw_dependency)
                .collect::<Vec<RustRawDependency>>();
            (p.id.repr.clone(), dependencies)
        })
        .collect()
}

pub fn resolve_rust_dependencies(
    metadata: &cargo_metadata::Metadata,
    packages: &[RustPackage],
) -> PackageIdToRustDependency {
    let nodes = get_nodes_from_metadata(metadata);

    packages
        .iter()
        .filter_map(|p| {
            let id = p.id.clone();
            find_node(&nodes, &id, "Skipping dependency.").map(|node| (id, node))
        })
        .map(|(id, node)| {
            let dependencies = node
                .deps
                .iter()
                .map(metadata_node_dep_to_rust_dependency)
                .collect::<Vec<RustDependency>>();
            (id, dependencies)
        })
        .collect()
}
