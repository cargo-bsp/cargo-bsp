//! Handles the upcoming requests from the client that does not require the
//! communication with Cargo (such as compile, run or test requests).

use log::warn;

use crate::project_model::sources::get_sources_for_target;
use crate::server::global_state::{GlobalState, GlobalStateSnapshot};
use crate::server::Result;

pub(crate) fn handle_workspace_build_targets(
    state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::bsp::WorkspaceBuildTargetsResult> {
    Ok(bsp_types::bsp::WorkspaceBuildTargetsResult {
        targets: state.workspace.get_bsp_build_targets(),
    })
}

pub(crate) fn handle_sources(
    state: GlobalStateSnapshot,
    params: bsp_types::bsp::SourcesParams,
) -> Result<bsp_types::bsp::SourcesResult> {
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

    Ok(bsp_types::bsp::SourcesResult {
        items: sources_items,
    })
}

// TODO: Not properly handled yet
pub(crate) fn handle_resources(
    _: GlobalStateSnapshot,
    _: bsp_types::bsp::ResourcesParams,
) -> Result<bsp_types::bsp::ResourcesResult> {
    Ok(bsp_types::bsp::ResourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_clean_cache(
    _: GlobalStateSnapshot,
    _: bsp_types::bsp::CleanCacheParams,
) -> Result<bsp_types::bsp::CleanCacheResult> {
    Ok(bsp_types::bsp::CleanCacheResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_dependency_modules(
    _: GlobalStateSnapshot,
    _: bsp_types::bsp::DependencyModulesParams,
) -> Result<bsp_types::bsp::DependencyModulesResult> {
    Ok(bsp_types::bsp::DependencyModulesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_dependency_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::bsp::DependencySourcesParams,
) -> Result<bsp_types::bsp::DependencySourcesResult> {
    Ok(bsp_types::bsp::DependencySourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_inverse_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::bsp::InverseSourcesParams,
) -> Result<bsp_types::bsp::InverseSourcesResult> {
    Ok(bsp_types::bsp::InverseSourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_output_paths(
    _: GlobalStateSnapshot,
    _: bsp_types::bsp::OutputPathsParams,
) -> Result<bsp_types::bsp::OutputPathsResult> {
    Ok(bsp_types::bsp::OutputPathsResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_workspace_libraries(
    _: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::bazel::WorkspaceLibrariesResult> {
    Ok(bsp_types::bazel::WorkspaceLibrariesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_workspace_directories(
    _: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::bazel::WorkspaceDirectoriesResult> {
    Ok(bsp_types::bazel::WorkspaceDirectoriesResult::default())
}

pub(crate) fn handle_reload(global_state: &mut GlobalState, _: ()) -> Result<()> {
    global_state.update_workspace_data();
    Ok(())
}

// BSP Cargo Extension handlers

pub(crate) fn handle_set_cargo_features(
    state: &mut GlobalState,
    params: bsp_types::cargo::SetCargoFeaturesParams,
) -> Result<bsp_types::cargo::SetCargoFeaturesResult> {
    let mutable_workspace = &mut state.workspace;
    let status_code =
        mutable_workspace.set_features_for_the_package(params.package_id, &params.features);
    Ok(bsp_types::cargo::SetCargoFeaturesResult { status_code })
}

pub(crate) fn handle_cargo_features_state(
    state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::cargo::CargoFeaturesStateResult> {
    let packages_features = state.workspace.get_cargo_features_state();

    Ok(bsp_types::cargo::CargoFeaturesStateResult { packages_features })
}
