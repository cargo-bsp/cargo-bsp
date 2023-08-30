//! This module ia an implementation of handling the BSP Rust extension.

mod dependency;
mod package;
mod target;
mod toolchain;

pub use self::package::get_rust_packages_related_to_targets;
pub use self::toolchain::get_rust_toolchains;

use crate::project_model::project_manifest::ProjectManifest;
use crate::project_model::rust_extension::dependency::{
    resolve_raw_dependencies, resolve_rust_dependencies,
};
use crate::project_model::workspace::ProjectWorkspace;
use crate::server::Error;
use bsp_types::extensions::{RustEdition, RustWorkspaceResult};
use bsp_types::BuildTargetIdentifier;
use cargo_metadata::{Edition, Metadata, Node};
use log::warn;

pub(crate) fn metadata_edition_to_rust_extension_edition(metadata_edition: Edition) -> RustEdition {
    match metadata_edition {
        Edition::E2015 => RustEdition::Edition2015,
        Edition::E2018 => RustEdition::Edition2018,
        _ => RustEdition::Edition2021,
    }
}

pub(crate) fn find_node<'a>(
    nodes: &'a [Node],
    package_id: &String,
    warning_not_found: &'static str,
) -> Option<&'a Node> {
    if let Some(n) = nodes.iter().find(|n| n.id.to_string() == *package_id) {
        Some(n)
    } else {
        warn!(
            "Couldn't find the node for {}. {}",
            package_id, warning_not_found
        );
        None
    }
}

pub(crate) fn get_nodes_from_metadata(metadata: &Metadata) -> Vec<Node> {
    match metadata.resolve.clone() {
        Some(resolve) => resolve.nodes,
        None => {
            warn!("Cargo metadata has no resolve field. Proceeding with an empty nodes vector.");
            vec![]
        }
    }
}

pub fn get_metadata(project_manifest: &ProjectManifest) -> Result<Metadata, Error> {
    let metadata = ProjectWorkspace::call_cargo_metadata_command(&project_manifest.file, true)?;
    Ok(metadata)
}

pub fn resolve_rust_workspace_result(
    workspace: &ProjectWorkspace,
    targets: &[BuildTargetIdentifier],
    metadata: &Metadata,
) -> RustWorkspaceResult {
    let packages = get_rust_packages_related_to_targets(workspace, metadata, targets);
    let raw_dependencies = resolve_raw_dependencies(metadata, &packages);
    let dependencies = resolve_rust_dependencies(metadata, &packages);

    RustWorkspaceResult {
        packages,
        raw_dependencies,
        dependencies,
        resolved_targets: Vec::from(targets),
    }
}
