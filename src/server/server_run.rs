// copy from rust-analyzer

//! Driver for rust-analyzer.
//!
//! Based on cli flags, either spawns an LSP server, or runs a batch analysis

#![warn(unused_lifetimes, semicolon_in_expressions_from_macros)]

use crate::bsp_types::requests::{InitializeBuildParams, InitializeBuildResult};
use crate::communication::Connection;
use crate::logger::log;
use crate::server;
use crate::server::{from_json, Result};

pub fn run_server() -> Result<()> {
    log("server will start");

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start()?;
    let initialize_params =
        from_json::<InitializeBuildParams>("InitializeParams", &initialize_params)?;
    log(&format!("InitializeParams: {:#?}", initialize_params));

    let initialize_result = InitializeBuildResult {
        display_name: "test".to_string(),
        version: "0.0.1".to_string(),
        bsp_version: "2.0.0".to_string(),
        capabilities: Default::default(),
        data: None,
    };

    let initialize_result = serde_json::to_value(initialize_result).unwrap();

    connection.initialize_finish(initialize_id, initialize_result)?;

    log(&format!(
        "Client '{}' {}",
        initialize_params.display_name, initialize_params.version
    ));

    server::main_loop(connection)?;

    io_threads.join()?;
    log("server did shut down");
    Ok(())
}
