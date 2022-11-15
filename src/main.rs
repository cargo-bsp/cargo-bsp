use std::process::{Child, Command, Stdio};

mod bsp_types;

fn example_client_initialize_query() -> bsp_types::InitializeBuildParams<String> {
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

// runs /src/server.rs binary which inherits I/O
fn spawn_server_process() -> Child {
    // Prepare to run `cargo run --bin server`
    let mut command = Command::new("cargo");
    command.arg("run").arg("--bin").arg("server");

    // Inherit I/O
    command.stdin(Stdio::inherit()).stdout(Stdio::inherit());

    // Run the command and return handle
    command.spawn().unwrap()
}

fn main() {
    println!("Starting server process");
    let mut server = spawn_server_process();
    println!(
        "Server process exited with status: {}",
        server.wait().unwrap()
    );

    // Note that serde parses Option(None) as null, does not skip it.
    println!(
        "\nPrinting example_client_initialize_query_json():{}",
        example_client_initialize_query_json()
    );
    println!("Hello, world!");
}
