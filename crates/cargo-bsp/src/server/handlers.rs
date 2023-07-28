//! Handles the upcoming requests from the client that does not require the
//! communication with Cargo (such as compile, run or test requests).

use log::warn;
use std::{ops::Deref, sync::Arc};

use crate::project_model::rust_extension::get_rust_toolchains;
use crate::project_model::sources::get_sources_for_target;
use crate::server::global_state::{GlobalState, GlobalStateSnapshot};
use crate::server::Result;
use bsp_types;

pub(crate) fn handle_workspace_build_targets(
    state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::requests::WorkspaceBuildTargetsResult> {
    Ok(bsp_types::requests::WorkspaceBuildTargetsResult {
        targets: state.workspace.get_bsp_build_targets(),
    })
}

pub(crate) fn handle_sources(
    state: GlobalStateSnapshot,
    params: bsp_types::requests::SourcesParams,
) -> Result<bsp_types::requests::SourcesResult> {
    let sources_items = params
        .targets
        .into_iter()
        .filter_map(|id| {
            state
                .workspace
                .get_target_details(&id)
                .or_else(|| {
                    warn!("Failed to get target details for: {:?}", id);
                    None
                })
                .map(|details| get_sources_for_target(&id, details))
        })
        .collect();

    Ok(bsp_types::requests::SourcesResult {
        items: sources_items,
    })
}

// TODO: Not properly handled yet
pub(crate) fn handle_resources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::ResourcesParams,
) -> Result<bsp_types::requests::ResourcesResult> {
    Ok(bsp_types::requests::ResourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_clean_cache(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::CleanCacheParams,
) -> Result<bsp_types::requests::CleanCacheResult> {
    Ok(bsp_types::requests::CleanCacheResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_dependency_modules(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::DependencyModulesParams,
) -> Result<bsp_types::requests::DependencyModulesResult> {
    Ok(bsp_types::requests::DependencyModulesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_dependency_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::DependencySourcesParams,
) -> Result<bsp_types::requests::DependencySourcesResult> {
    Ok(bsp_types::requests::DependencySourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_inverse_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::InverseSourcesParams,
) -> Result<bsp_types::requests::InverseSourcesResult> {
    Ok(bsp_types::requests::InverseSourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_output_paths(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::OutputPathsParams,
) -> Result<bsp_types::requests::OutputPathsResult> {
    Ok(bsp_types::requests::OutputPathsResult::default())
}

pub(crate) fn handle_reload(global_state: &mut GlobalState, _: ()) -> Result<()> {
    global_state.update_workspace_data();
    Ok(())
}

// BSP Cargo Extension handlers

pub(crate) fn handle_disable_cargo_features(
    state: &mut GlobalState,
    params: bsp_types::extensions::DisableCargoFeaturesParams,
) -> Result<()> {
    let mutable_workspace = Arc::make_mut(&mut state.workspace);
    mutable_workspace.disable_features_for_package(params.package_id, &params.features);
    Ok(())
}

pub(crate) fn handle_enable_cargo_features(
    state: &mut GlobalState,
    params: bsp_types::extensions::EnableCargoFeaturesParams,
) -> Result<()> {
    let mutable_workspace = Arc::make_mut(&mut state.workspace);
    mutable_workspace.enable_features_for_package(params.package_id, &params.features);
    Ok(())
}

pub(crate) fn handle_cargo_features_state(
    state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::extensions::CargoFeaturesStateResult> {
    let packages_features = state.workspace.get_cargo_features_state();

    Ok(bsp_types::extensions::CargoFeaturesStateResult { packages_features })
}

// BSP Rust extension handlers

pub(crate) fn handle_rust_toolchain(
    state: GlobalStateSnapshot,
    params: bsp_types::extensions::RustToolchainParams,
) -> Result<bsp_types::extensions::RustToolchainResult> {
    Ok(bsp_types::extensions::RustToolchainResult {
        toolchains: get_rust_toolchains(state.workspace.deref(), params.targets),
    })
}
