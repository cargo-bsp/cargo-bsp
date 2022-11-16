use std::io::prelude::*;
use std::io::stdin;

use crate::bsp_types::{BuildServerCapabilities, InitializeBuildParams, InitializeBuildResult, RequestRPC, ResponseRPC};
use crate::utils::{log, send};

fn example_server_response() -> InitializeBuildResult {
    InitializeBuildResult {
        display_name: "test1".to_string(),
        version: "test2".to_string(),
        bsp_version: "test3".to_string(),
        capabilities: BuildServerCapabilities {
            compile_provider: None,
            test_provider: None,
            run_provider: None,
            debug_provider: None,
            inverse_sources_provider: None,
            dependency_sources_provider: None,
            dependency_modules_provider: None,
            resources_provider: None,
            output_paths_provider: None,
            build_target_changed_provider: None,
            jvm_run_environment_provider: None,
            jvm_test_environment_provider: None,
            can_reload: None,
        },
        data: None,
    }
}

pub fn run_server() {
    log("Server has started\n");

    let response_string = example_server_response().parse_to_string();
    log(&format!("Basic response: {}\n", response_string));

    for line in stdin().lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }
        match InitializeBuildParams::parse_from_string(&line_string) {
            Ok(r) => {
                log(&format!("Received proper request from client: {:?}\n", r));
                send(&response_string);
            }
            Err(_) => {
                log(&format!("Received some string from client: {}\n", line_string));
            }
        }
    }
}
