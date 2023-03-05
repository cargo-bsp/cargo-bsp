use crate::bsp_types;
use crate::bsp_types::notifications::StatusCode;
use crate::server::global_state::GlobalState;
use crate::server::request_actor::RequestHandle;
use crate::bsp_types::requests::{CompileParams, CreateCommand, RunParams, TestParams};
use crate::communication::RequestId;
use crate::server::Error;

pub(crate) fn handle_workspace_build_targets(
    _: &mut GlobalState,
    _: (),
) -> Result<bsp_types::requests::WorkspaceBuildTargetsResult, Error> {
    let result = bsp_types::requests::WorkspaceBuildTargetsResult {
        targets: vec![bsp_types::BuildTarget {
            id: bsp_types::BuildTargetIdentifier {
                uri: "//:hello".to_string(),
            },
            display_name: Some("//:hello".to_string()),
            base_directory: None,
            tags: vec![],
            capabilities: bsp_types::BuildTargetCapabilities {
                can_compile: true,
                can_test: true,
                can_run: true,
                can_debug: false,
            },
            language_ids: vec![],
            dependencies: vec![],
            data_kind: None,
            data: None,
        }],
    };
    Ok(result)
}

pub(crate) fn handle_sources(
    _: &mut GlobalState,
    _: bsp_types::requests::SourcesParams,
) -> Result<bsp_types::requests::SourcesResult, Error> {
    Ok(bsp_types::requests::SourcesResult::default())
}

pub(crate) fn handle_resources(
    _: &mut GlobalState,
    _: bsp_types::requests::ResourcesParams,
) -> Result<bsp_types::requests::ResourcesResult, Error> {
    Ok(bsp_types::requests::ResourcesResult::default())
}

pub(crate) fn handle_extensions(
    _: &mut GlobalState,
    _: bsp_types::requests::JavacOptionsParams,
) -> Result<bsp_types::requests::JavacOptionsResult, Error> {
    Ok(bsp_types::requests::JavacOptionsResult::default())
}

pub(crate) fn handle_compile(
    global_state: &mut GlobalState,
    params: CompileParams,
) -> Result<bsp_types::requests::CompileResult, Error> {

    // global_state.send_notification::<bsp_types::notifications::LogMessage>(
    //     bsp_types::notifications::LogMessageParams {
    //         message_type: bsp_types::notifications::MessageType::Log,
    //         task: None,
    //         origin_id: params.origin_id.clone(),
    //         message: "INFO: Build completed successfully".to_string(),
    //     },
    // );
    let (sender_to_main, _) = global_state.threads_chan.clone();
    let _request_handle = RequestHandle::spawn(
        Box::new(move |msg| sender_to_main.send(msg).unwrap()),
        RequestId::from("TODO".to_owned()),
        Box::new(params) as Box<dyn CreateCommand + Send>,
    );
    // TODO add the actor to map ReqToActor ~ Kasia

    let result = bsp_types::requests::CompileResult {
        origin_id: None,
        status_code: 1,
        data_kind: None,
        data: None,
    };
    Ok(result)
}

pub(crate) fn handle_run(
    global_state: &mut GlobalState,
    params: RunParams,
) -> Result<bsp_types::requests::RunResult, Error> {
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
    params: TestParams,
) -> Result<bsp_types::requests::TestResult, Error> {
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

pub(crate) fn handle_reload(_: &mut GlobalState, _: ()) -> Result<(), Error> {
    Ok(())
}
