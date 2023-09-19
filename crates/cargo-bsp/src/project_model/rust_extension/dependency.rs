//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are used to resolve the dependencies part of the request.

use std::collections::BTreeMap;

use cargo_metadata::DependencyKind;

use bsp_types::extensions::{
    Feature, RustDepKind, RustDepKindInfo, RustDependencies, RustDependency, RustPackage,
    RustRawDependencies, RustRawDependency,
};

use crate::project_model::rust_extension::{find_node, get_nodes_from_metadata};

fn package_dependency_to_rust_raw_dependency(
    package_dependency: cargo_metadata::Dependency,
) -> RustRawDependency {
    let kind = match package_dependency.kind {
        // Cargo metadata output defaults to Null, when dependency is normal. Since we want here
        // to have the same behavior as cargo metadata, we return None for normal dependency kind.
        // Unclassified kind should also return None (Null).
        DependencyKind::Normal | DependencyKind::Unknown => None,
        _ => Some(metadata_dep_kind_to_rust_dep_kind(package_dependency.kind)),
    };
    RustRawDependency {
        name: package_dependency.name,
        optional: package_dependency.optional,
        uses_default_features: package_dependency.uses_default_features,
        features: package_dependency
            .features
            .into_iter()
            .map(Feature)
            .collect(),
        rename: package_dependency.rename,
        kind,
        target: package_dependency.target.map(|p| p.to_string()),
    }
}

fn metadata_dep_kind_to_rust_dep_kind(metadata_dep_kind: DependencyKind) -> RustDepKind {
    match metadata_dep_kind {
        DependencyKind::Build => RustDepKind::BUILD,
        DependencyKind::Development => RustDepKind::DEV,
        DependencyKind::Normal => RustDepKind::NORMAL,
        DependencyKind::Unknown => RustDepKind::UNCLASSIFIED,
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
) -> RustRawDependencies {
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
        .collect::<BTreeMap<String, Vec<RustRawDependency>>>()
        .into()
}

pub fn resolve_rust_dependencies(
    metadata: &cargo_metadata::Metadata,
    packages: &[RustPackage],
) -> RustDependencies {
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
        .collect::<BTreeMap<String, Vec<RustDependency>>>()
        .into()
}
