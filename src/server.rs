use std::io::prelude::*;
use std::io::stdin;

use jsonrpsee_server::RpcModule;

use crate::bsp_types::{InitializeBuildParams, InitializeBuildResult, MethodName};
use crate::utils::{log, send};

pub struct Server {
    module: RpcModule<()>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            module: Server::create_module(),
        }
    }

    fn create_module() -> RpcModule<()> {
        let mut module = RpcModule::new(());
        module
            .register_method(InitializeBuildParams::get_method(), |params, _| {
                params
                    .parse::<InitializeBuildParams>()
                    .map::<InitializeBuildResult<()>, _>(|_| InitializeBuildResult::default())
                    .map_err(Into::into)
            })
            .unwrap();
        module
    }

    async fn handle(&self, request_string: &str) {
        let (resp, _) = self.module.raw_json_request(request_string).await.unwrap();
        if resp.success {
            log(&format!(
                "Received proper request from client: {:}\n",
                &request_string
            ));
            log(&format!("Responding to client with: {:}\n", &resp.result));
            send(&resp.result);
        } else {
            log(&format!(
                "Received some string from client: {}\n response: {}\n",
                request_string, resp.result
            ));
        }
    }

    pub async fn run(&mut self) {
        log("Server has started\n");

        for line in stdin().lock().lines() {
            let line_string = line.unwrap();

            if line_string.is_empty() {
                break;
            }

            self.handle(&line_string).await;
        }
    }
}
