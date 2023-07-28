//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are used to resolve the dependencies part of the request.

use crate::project_model::package_dependency::PackageDependency;
use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::{
    PackageIdToRustRawDependency, PackageSourceToRustDependency, RustDepKind, RustDepKindInfo,
    RustDependency, RustRawDependency,
};
use bsp_types::BuildTargetIdentifier;
use cargo_metadata::DependencyKind;
use log::warn;

fn metadata_dependency_kind_to_string(metadata_dependency_kind: DependencyKind) -> Option<String> {
    match metadata_dependency_kind {
        DependencyKind::Build => Some("build".to_string()),
        DependencyKind::Development => Some("dev".to_string()),
        DependencyKind::Normal => None, // Cargo metadata output defaults to Null, when dependency is normal
        _ => None,
    }
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
        name: node_dep.name.clone(),
        target: node_dep.pkg.to_string(),
        dep_kinds: metadata_dep_kinds_info_to_rust_dep_kinds_info(&node_dep.dep_kinds),
    }
}

pub fn resolve_raw_dependencies(
    workspace: &ProjectWorkspace,
    targets: &[BuildTargetIdentifier],
) -> PackageIdToRustRawDependency {
    workspace
        .get_packages_related_to_targets(targets)
        .iter()
        .flat_map(|p| {
            p.dependencies
                .iter()
                .cloned()
                .map(|d| (p.id.clone(), package_dependency_to_rust_raw_dependency(d)))
                .collect::<Vec<(String, RustRawDependency)>>()
        })
        .collect()
}

pub fn resolve_rust_dependencies(
    workspace: &ProjectWorkspace,
    metadata: &cargo_metadata::Metadata,
    targets: &[BuildTargetIdentifier],
) -> PackageSourceToRustDependency {
    let nodes = if let Some(resolve) = metadata.resolve.clone() {
        resolve.nodes
    } else {
        warn!("No resolve field in cargo metadata. Returning default value for rust dependencies");
        return Default::default();
    };

    workspace
        .get_packages_related_to_targets(targets)
        .iter()
        .filter_map(|p| {
            let id = p.id.clone();
            if let Some(n) = nodes.iter().find(|n| n.id.to_string() == *id) {
                Some((id, n))
            } else {
                warn!(
                    "No node found in cargo metadata for the {}. Skipping it.",
                    id
                );
                None
            }
        })
        .flat_map(|(id, node)| {
            node.deps
                .iter()
                .map(move |d| (id.clone(), metadata_node_dep_to_rust_dependency(d)))
        })
        .collect()
}
