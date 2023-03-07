// use std::sync::Arc;
use crate::bsp_types;
use crate::bsp_types::notifications::StatusCode;
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

pub(crate) fn handle_compile(
    global_state: &mut GlobalState,
    params: bsp_types::requests::CompileParams,
) -> Result<bsp_types::requests::CompileResult> {
    global_state.send_notification::<bsp_types::notifications::LogMessage>(
        bsp_types::notifications::LogMessageParams {
            message_type: bsp_types::notifications::MessageType::Log,
            task: None,
            origin_id: params.origin_id.clone(),
            message: "INFO: Build completed successfully".to_string(),
        },
    );
    let result = bsp_types::requests::CompileResult {
        origin_id: params.origin_id,
        status_code: 1,
        data_kind: None,
        data: None,
    };
    Ok(result)
}

pub(crate) fn handle_run(
    global_state: &mut GlobalState,
    params: bsp_types::requests::RunParams,
) -> Result<bsp_types::requests::RunResult> {
    global_state.send_notification::<bsp_types::notifications::LogMessage>(
        bsp_types::notifications::LogMessageParams {
            message_type: bsp_types::notifications::MessageType::Log,
            task: None,
            origin_id: params.origin_id.clone(),
            message: "INFO: Run completed successfully".to_string(),
        },
    );
    let result = bsp_types::requests::RunResult {
        origin_id: params.origin_id,
        status_code: 1,
    };
    Ok(result)
}

pub(crate) fn handle_test(
    global_state: &mut GlobalState,
    params: bsp_types::requests::TestParams,
) -> Result<bsp_types::requests::TestResult> {
    global_state.send_notification::<bsp_types::notifications::LogMessage>(
        bsp_types::notifications::LogMessageParams {
            message_type: bsp_types::notifications::MessageType::Log,
            task: None,
            origin_id: params.origin_id.clone(),
            message: "INFO: Test completed successfully".to_string(),
        },
    );
    global_state.send_notification::<bsp_types::notifications::TaskFinish>(
        bsp_types::notifications::TaskFinishParams {
            task_id: Default::default(),
            event_time: None,
            message: None,
            status: StatusCode::Ok,
            data: None,
        },
    );
    let result = bsp_types::requests::TestResult {
        origin_id: params.origin_id,
        status_code: 1,
        data_kind: None,
        data: None,
    };
    Ok(result)
}

pub(crate) fn handle_reload(global_state: &mut GlobalState, _: ()) -> Result<()> {
    global_state.update_workspace_data();
    Ok(())
}
