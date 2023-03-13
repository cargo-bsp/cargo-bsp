// copy from rust-analyzer

//! Driver for rust-analyzer.
//!
//! Based on cli flags, either spawns an LSP server, or runs a batch analysis

#![warn(unused_lifetimes, semicolon_in_expressions_from_macros)]

use std::env;

use url::Url;

use crate::bsp_types::requests::{InitializeBuildParams, InitializeBuildResult};
use crate::communication::Connection;
use crate::logger::log;
use crate::project_model::ProjectManifest;
use crate::server;
use crate::server::{from_json, Result};
use crate::server::caps::server_capabilities;
use crate::server::config::Config;

pub fn run_server() -> Result<()> {
    log("server will start");

    let (connection, io_threads) = Connection::stdio();

    let (initialize_id, initialize_params) = connection.initialize_start()?;
    let initialize_params =
        from_json::<InitializeBuildParams>("InitializeParams", &initialize_params)?;

    let root_path = match Url::try_from(initialize_params.root_uri.as_str())
        .ok()
        .and_then(|it| it.to_file_path().ok())
    {
        Some(it) => it,
        None => env::current_dir()?,
    };

    let mut config = Config::new(root_path, initialize_params.capabilities);
    let server_capabilities = server_capabilities(&config);

    let initialize_result = InitializeBuildResult {
        display_name: "test".to_string(),
        version: "0.0.1".to_string(),
        bsp_version: "2.0.0".to_string(),
        capabilities: server_capabilities,
        data: None,
    };

    let initialize_result = serde_json::to_value(initialize_result).unwrap();

    connection.initialize_finish(initialize_id, initialize_result)?;

    if config.linked_projects().is_empty() {
        let discovered = ProjectManifest::discover_all(config.root_path());
        if discovered.is_empty() {
            log(&format!(
                "error: failed to find any projects in {:?}",
                config.root_path()
            ));
        }
        config.discovered_projects = discovered;
    }

    server::main_loop(config, connection)?;

    io_threads.join()?;
    log("server did shut down");
    Ok(())
}
