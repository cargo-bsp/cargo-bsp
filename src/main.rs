mod bsp_types;
mod server;

use crate::server::run_server;

fn example_client_initialize_query()-> bsp_types::InitializeBuildParams<String> {
    bsp_types::InitializeBuildParams {
        display_name: "rust-bsp-client".to_string(),
        version: "0.1.0".to_string(),
        bsp_version: "2.0.0-M5".to_string(),
        root_uri: "file:///home/krzysztof/Projects/rust-bsp-client".to_string(),
        capabilities: bsp_types::BuildClientCapabilities {
            language_ids: vec!["rust".to_string()],
        },
        data: None,
    }
}

fn example_client_initialize_query_json() -> String {
    serde_json::to_string(&example_client_initialize_query()).unwrap()
}

fn main() {
    // Note that serde parses Option(None) as null, does not skip it.
    println!("{}", example_client_initialize_query_json());
    println!("Hello, world!");
    run_server();
}
