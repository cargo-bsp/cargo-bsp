//! Handles the upcoming requests from the client that does not require the
//! communication with Cargo (such as compile, run or test requests).

use log::warn;

use crate::project_model::sources::get_sources_for_target;
use crate::server::global_state::{GlobalState, GlobalStateSnapshot};
use crate::server::Result;

pub(crate) fn handle_workspace_build_targets(
    state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp4rs::bsp::WorkspaceBuildTargetsResult> {
    Ok(bsp4rs::bsp::WorkspaceBuildTargetsResult {
        targets: state.workspace.get_bsp_build_targets(),
    })
}

pub(crate) fn handle_sources(
    state: GlobalStateSnapshot,
    params: bsp4rs::bsp::SourcesParams,
) -> Result<bsp4rs::bsp::SourcesResult> {
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

    Ok(bsp4rs::bsp::SourcesResult {
        items: sources_items,
    })
}

// TODO: Not properly handled yet
pub(crate) fn handle_resources(
    _: GlobalStateSnapshot,
    _: bsp4rs::bsp::ResourcesParams,
) -> Result<bsp4rs::bsp::ResourcesResult> {
    Ok(bsp4rs::bsp::ResourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_clean_cache(
    _: GlobalStateSnapshot,
    _: bsp4rs::bsp::CleanCacheParams,
) -> Result<bsp4rs::bsp::CleanCacheResult> {
    Ok(bsp4rs::bsp::CleanCacheResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_dependency_modules(
    _: GlobalStateSnapshot,
    _: bsp4rs::bsp::DependencyModulesParams,
) -> Result<bsp4rs::bsp::DependencyModulesResult> {
    Ok(bsp4rs::bsp::DependencyModulesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_dependency_sources(
    _: GlobalStateSnapshot,
    _: bsp4rs::bsp::DependencySourcesParams,
) -> Result<bsp4rs::bsp::DependencySourcesResult> {
    Ok(bsp4rs::bsp::DependencySourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_inverse_sources(
    _: GlobalStateSnapshot,
    _: bsp4rs::bsp::InverseSourcesParams,
) -> Result<bsp4rs::bsp::InverseSourcesResult> {
    Ok(bsp4rs::bsp::InverseSourcesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_output_paths(
    _: GlobalStateSnapshot,
    _: bsp4rs::bsp::OutputPathsParams,
) -> Result<bsp4rs::bsp::OutputPathsResult> {
    Ok(bsp4rs::bsp::OutputPathsResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_workspace_libraries(
    _: GlobalStateSnapshot,
    _: (),
) -> Result<bsp4rs::bazel::WorkspaceLibrariesResult> {
    Ok(bsp4rs::bazel::WorkspaceLibrariesResult::default())
}

// TODO: Not properly handled yet
pub(crate) fn handle_workspace_directories(
    _: GlobalStateSnapshot,
    _: (),
) -> Result<bsp4rs::bazel::WorkspaceDirectoriesResult> {
    Ok(bsp4rs::bazel::WorkspaceDirectoriesResult::default())
}

pub(crate) fn handle_reload(global_state: &mut GlobalState, _: ()) -> Result<()> {
    global_state.update_workspace_data();
    Ok(())
}

// BSP Cargo Extension handlers

pub(crate) fn handle_set_cargo_features(
    state: &mut GlobalState,
    params: bsp4rs::cargo::SetCargoFeaturesParams,
) -> Result<bsp4rs::cargo::SetCargoFeaturesResult> {
    let mutable_workspace = &mut state.workspace;
    let status_code =
        mutable_workspace.set_features_for_the_package(params.package_id, &params.features);
    Ok(bsp4rs::cargo::SetCargoFeaturesResult { status_code })
}

pub(crate) fn handle_cargo_features_state(
    state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp4rs::cargo::CargoFeaturesStateResult> {
    let packages_features = state.workspace.get_cargo_features_state();

    Ok(bsp4rs::cargo::CargoFeaturesStateResult { packages_features })
}
