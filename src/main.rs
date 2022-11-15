use std::io::{stderr, Write, stdin, stdout};
use std::io::prelude::*;
use nix::unistd::{dup2, pipe, fork, ForkResult};
use crate::bsp_types::{BuildClientCapabilities, InitializeBuildParams, RequestRPC};

mod server;

use crate::server::run_server;

mod bsp_types;

#[allow(unused)]
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

#[allow(unused)]
fn example_client_initialize_query_json() -> String {
    serde_json::to_string(&example_client_initialize_query()).unwrap()
}

fn run_client() {
    stderr().write_all("Client started\n".as_bytes()).unwrap();
    println!("Hello, it's me - client :>");

    let request_string = example_client_initialize_query().parse_to_string() + "\n";
    let msg = format!("Basic request: {}", request_string);
    stderr().write_all(msg.as_bytes()).unwrap();

    stdout().write_all(request_string.as_bytes()).unwrap();

    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }

        let msg = format!("Received message from server: {}\n", line_string);
        stderr().write_all(msg.as_bytes()).unwrap();
    }
}

fn main() {
    let server_to_client = pipe().unwrap();
    let client_to_server = pipe().unwrap();

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child: _child, .. }) => {
                dup2(server_to_client.0, 0).unwrap();
                dup2(client_to_server.1, 1).unwrap();
                run_client();
            }
            Ok(ForkResult::Child) => {
                dup2(client_to_server.0, 0).unwrap();
                dup2(server_to_client.1, 1).unwrap();
                run_server();
            }
            Err(_) => println!("Fork failed"),
        }
    }
}
