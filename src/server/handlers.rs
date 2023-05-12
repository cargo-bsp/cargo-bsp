use crate::bsp_types;
use crate::server::global_state::{GlobalState, GlobalStateSnapshot};
use crate::server::Result;

pub(crate) fn handle_workspace_build_targets(
    global_state: GlobalStateSnapshot,
    _: (),
) -> Result<bsp_types::requests::WorkspaceBuildTargetsResult> {
    Ok(bsp_types::requests::WorkspaceBuildTargetsResult {
        targets: global_state.workspace.get_bsp_build_targets(),
    })
}

pub(crate) fn handle_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::SourcesParams,
) -> Result<bsp_types::requests::SourcesResult> {
    Ok(bsp_types::requests::SourcesResult::default())
}

pub(crate) fn handle_resources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::ResourcesParams,
) -> Result<bsp_types::requests::ResourcesResult> {
    Ok(bsp_types::requests::ResourcesResult::default())
}

// the current version of the client sends a java extension request even though we are not working with java.
// to be removed once it is fixed in the client
pub(crate) fn handle_java_extensions(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::JavacOptionsParams,
) -> Result<bsp_types::requests::JavacOptionsResult> {
    Ok(bsp_types::requests::JavacOptionsResult::default())
}

pub(crate) fn handle_clean_cache(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::CleanCacheParams,
) -> Result<bsp_types::requests::CleanCacheResult> {
    Ok(bsp_types::requests::CleanCacheResult::default())
}

pub(crate) fn handle_dependency_modules(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::DependencyModulesParams,
) -> Result<bsp_types::requests::DependencyModulesResult> {
    Ok(bsp_types::requests::DependencyModulesResult::default())
}

pub(crate) fn handle_dependency_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::DependencySourcesParams,
) -> Result<bsp_types::requests::DependencySourcesResult> {
    Ok(bsp_types::requests::DependencySourcesResult::default())
}

pub(crate) fn handle_inverse_sources(
    _: GlobalStateSnapshot,
    _: bsp_types::requests::InverseSourcesParams,
) -> Result<bsp_types::requests::InverseSourcesResult> {
    Ok(bsp_types::requests::InverseSourcesResult::default())
}

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
