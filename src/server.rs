use std::io::prelude::*;
use std::io::stdin;

use jsonrpsee_server::RpcModule;

use crate::bsp_types::{InitializeBuildParams, InitializeBuildResult};
use crate::utils::{log, send};

fn create_module() -> RpcModule<()> {
    let mut module = RpcModule::new(());
    module
        .register_method("build/initialize", |params, _| {
            params
                .parse::<InitializeBuildParams>()
                .map::<InitializeBuildResult<()>, _>(|_| InitializeBuildResult::default())
                .map_err(Into::into)
        })
        .unwrap();
    module
}

pub async fn run_server() {
    log("Server has started\n");

    let module = create_module();

    for line in stdin().lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }
        let (resp, _) = module.raw_json_request(&line_string).await.unwrap();
        if resp.success {
            log(&format!(
                "Received proper request from client: {:}\n",
                &resp.result
            ));
            send(&resp.result);
        } else {
            log(&format!(
                "Received some string from client: {}\n response: {}\n",
                line_string, resp.result
            ));
        }
    }
}
