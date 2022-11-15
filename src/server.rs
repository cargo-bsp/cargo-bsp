// You can run this with `cargo run --bin server

use std::io;
use std::io::{stderr, stdout};
use std::io::prelude::*;


use crate::bsp_types::{BuildServerCapabilities, InitializeBuildParams, InitializeBuildResult, RequestRPC, ResponseRPC};

#[allow(unused)]
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
    stderr().write_all("Server has started\n".as_bytes()).unwrap();

    let response_string = example_server_response().parse_to_string() + "\n";
    let msg = format!("Basic response: {}", response_string);
    stderr().write_all(msg.as_bytes()).unwrap();

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }

        let request = InitializeBuildParams::parse_from_string(&line_string);
        match request {
            Ok(r) => {
                let msg = format!("Received proper request from client: {:?}\n", r);
                stderr().write_all(msg.as_bytes()).unwrap();
                stdout().write_all(response_string.as_bytes()).unwrap()
            }
            Err(_) => {
                let msg = format!("Received some string from client: {}\n", line_string);
                stderr().write_all(msg.as_bytes()).unwrap();
            }
        }
    }
}