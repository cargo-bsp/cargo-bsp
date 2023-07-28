//! This module ia an implementation of handling the BSP Rust extension.

mod dependency;
mod package;
mod target;
mod toolchain;

pub use self::dependency::{resolve_raw_dependencies, resolve_rust_dependencies};
pub use self::package::get_rust_packages_related_to_targets;
pub use self::toolchain::get_rust_toolchains;

use crate::project_model::project_manifest::ProjectManifest;
use crate::project_model::workspace::ProjectWorkspace;
use crate::server::Error;
use bsp_types::extensions::{RustEdition, RustWorkspaceResult};
use bsp_types::BuildTargetIdentifier;
use cargo_metadata::Edition;

pub(crate) fn metadata_edition_to_rust_extension_edition(metadata_edition: Edition) -> RustEdition {
    match metadata_edition {
        Edition::E2015 => RustEdition::Edition2015,
        Edition::E2018 => RustEdition::Edition2018,
        _ => RustEdition::Edition2021,
    }
}

pub fn resolve_rust_workspace_result(
    workspace: &ProjectWorkspace,
    project_manifest: &ProjectManifest,
    targets: &[BuildTargetIdentifier],
) -> Result<RustWorkspaceResult, Error> {
    let metadata = ProjectWorkspace::call_cargo_metadata_command(&project_manifest.file)?;

    let packages = get_rust_packages_related_to_targets(workspace, &metadata, targets);
    let raw_dependencies = resolve_raw_dependencies(workspace, targets);
    let dependencies = resolve_rust_dependencies(workspace, &metadata, targets);

    Ok(RustWorkspaceResult {
        packages,
        raw_dependencies,
        dependencies,
        resolved_targets: Vec::new(), //Todo this is for Bazel
    })
}
