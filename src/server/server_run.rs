use std::env;

use bsp_server::Connection;
use log::info;
use url::Url;

use crate::bsp_types::requests::{InitializeBuildParams, InitializeBuildResult};
use crate::server;
use crate::server::caps::server_capabilities;
use crate::server::config::Config;
use crate::server::{from_json, Result};

pub fn run_server() -> Result<()> {
    info!("server will start");

    let (connection, io_threads) = Connection::bsp_stdio();

    let (initialize_id, initialize_params) = connection.bsp_initialize_start()?;
    let initialize_params =
        from_json::<InitializeBuildParams>("InitializeParams", &initialize_params)?;

    let root_path = match Url::try_from(initialize_params.root_uri.as_str())
        .ok()
        .and_then(|it| it.to_file_path().ok())
    {
        Some(it) => it,
        None => env::current_dir()?,
    };

    let config = Config::new(root_path, initialize_params.capabilities);
    let server_capabilities = server_capabilities(&config);

    let initialize_result = InitializeBuildResult {
        display_name: "test".to_string(),
        version: "0.0.1".to_string(),
        bsp_version: "2.0.0".to_string(),
        capabilities: server_capabilities,
        data: None,
    };

    let initialize_result = serde_json::to_value(initialize_result).unwrap();

    connection.bsp_initialize_finish(initialize_id, initialize_result)?;

    server::main_loop(config, connection)?;

    io_threads.join()?;
    info!("server did shut down");
    Ok(())
}
