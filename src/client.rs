use std::io::prelude::*;
use std::io::stdin;

use crate::bsp_types::{BuildClientCapabilities, InitializeBuildParams, RequestRPC};
use crate::utils::{log, send};

fn example_client_initialize_query() -> InitializeBuildParams {
    InitializeBuildParams {
        display_name: "rust-bsp-client".to_string(),
        version: "0.1.0".to_string(),
        bsp_version: "2.0.0-M5".to_string(),
        root_uri: "file:///home/krzysztof/Projects/rust-bsp-client".to_string(),
        capabilities: BuildClientCapabilities {
            language_ids: vec!["rust".to_string()],
        },
        data: None,
    }
}

pub fn run_client() {
    log("Client started\n");

    let request_string = example_client_initialize_query().parse_to_string();
    log(&format!("Basic request: {}\n", request_string));

    send(&request_string);

    for line in stdin().lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }

        log(&format!("Received message from server: {}\n", line_string));
    }
}
