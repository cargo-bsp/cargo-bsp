// use std::sync::Arc;
use crate::bsp_types;
use crate::bsp_types::notifications::StatusCode;
use crate::server::global_state::GlobalState;
use crate::server::Result;
use cargo_metadata::{MetadataCommand, CargoOpt, Target};
use crate::bsp_types::{BuildTarget, BuildTargetCapabilities};
use crate::logger::log;
use crate::project_model::ProjectManifest;


fn get_targets_from_project_manifest(project_manifest: &ProjectManifest) -> Vec<Target> {
    let path = project_manifest.file.clone();

    let metadata = MetadataCommand::new()
        .manifest_path(path)
        .features(CargoOpt::AllFeatures)
        .exec()
        .unwrap();

    metadata.workspace_packages()
        .iter()
        .flat_map(|package| package.targets.iter())
        .cloned()
        .collect()
}

fn create_bsp_build_targets_from_cargo_targets(cargo_targets: Vec<Target>) -> Vec<BuildTarget> {
    let mut build_targets = Vec::new();

    for cargo_target in cargo_targets.iter() {
        let capabilities = BuildTargetCapabilities {
            can_compile: true,
            can_test: cargo_target.test,
            can_run: true,
            can_debug: false,
        };

        let build_target = BuildTarget {
            id: bsp_types::BuildTargetIdentifier {
                uri: cargo_target.src_path.to_string(),
            },
            display_name: Some(cargo_target.name.clone()),
            base_directory: None, //Some(cargo_target.src_path.to_string()),
            tags: vec![],
            capabilities,
            language_ids: vec![],
            dependencies: vec![],
            data_kind: None,
            data: None,
        };
        build_targets.push(build_target);
    }

    build_targets

}

pub(crate) fn handle_workspace_build_targets(
    global_state: &mut GlobalState,
    _: (),
) -> Result<bsp_types::requests::WorkspaceBuildTargetsResult> {


    let hard_coded_targets = vec![bsp_types::BuildTarget {
        id: bsp_types::BuildTargetIdentifier {
            uri: "//:hello".to_string(),
        },
        display_name: Some("//:hello".to_string()),
        base_directory: None,
        tags: vec![],
        capabilities: bsp_types::BuildTargetCapabilities::default(),
        ..BuildTarget::default()
    }];

    let mut result = bsp_types::requests::WorkspaceBuildTargetsResult {
        targets: hard_coded_targets,
    };

    let mut project_manifests = global_state.config.discovered_projects.iter().flatten();

    // One workspace only
    if let Some(project_manifest) = project_manifests.next() {
        let cargo_targets = get_targets_from_project_manifest(project_manifest);
        // BuildTarget::from_cargo_targets(cargo_targets); // <- docelowo
        result.targets = create_bsp_build_targets_from_cargo_targets(cargo_targets);
    } else {
        log("No project manifests found!");
    }

    // Many workspaces
    // log("iterate over discovered_projects");
    // for project_manifest in global_state.config.discovered_projects.iter().flatten() {
    //
    //     let targets = get_targets_from_project_manifest(project_manifest);
    //     log(format!("targets: {:?}", targets).as_str());
    //
    // }
    Ok(result)
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

pub(crate) fn handle_reload(_: &mut GlobalState, _: ()) -> Result<()> {
    Ok(())
}
