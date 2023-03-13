use crate::bsp_types;
use crate::server::global_state::GlobalState;
use crate::server::Result;

pub(crate) fn handle_workspace_build_targets(
    global_state: &mut GlobalState,
    _: (),
) -> Result<bsp_types::requests::WorkspaceBuildTargetsResult> {
    Ok(bsp_types::requests::WorkspaceBuildTargetsResult {
        targets: global_state.workspace.build_targets.clone(),
    })
}

pub(crate) fn handle_sources(
    _: &mut GlobalState,
    _: bsp_types::requests::SourcesParams,
) -> Result<bsp_types::requests::SourcesResult> {
    Ok(bsp_types::requests::SourcesResult::default())
}

pub(crate) fn handle_resources(
    _: &mut GlobalState,
    _: bsp_types::requests::ResourcesParams,
) -> Result<bsp_types::requests::ResourcesResult> {
    Ok(bsp_types::requests::ResourcesResult::default())
}

pub(crate) fn handle_extensions(
    _: &mut GlobalState,
    _: bsp_types::requests::JavacOptionsParams,
) -> Result<bsp_types::requests::JavacOptionsResult> {
    Ok(bsp_types::requests::JavacOptionsResult::default())
}

pub(crate) fn handle_reload(_: &mut GlobalState, _: ()) -> Result<()> {
    Ok(())
}
