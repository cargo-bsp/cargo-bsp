use std::io::prelude::*;
use std::io::{stderr, stdin, Write};

use jsonrpsee_server::RpcModule;

use cargo_bsp::bsp_types::{InitializeBuildParams, InitializeBuildResult, MethodName};

pub struct Server {
    module: RpcModule<()>,
}

impl Default for Server {
    fn default() -> Self {
        Self {
            module: RpcModule::new(()),
        }
    }
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
            .register_method(InitializeBuildParams::get_method_name(), |params, _| {
                params
                    .parse::<InitializeBuildParams>()
                    .map::<InitializeBuildResult, _>(|_| InitializeBuildResult::default())
                    .map_err(Into::into)
            })
            .unwrap();
        module
    }

    async fn handle(&self, request_string: &str) {
        let (resp, _) = self.module.raw_json_request(request_string).await.unwrap();
        if resp.success {
            self.log(&format!(
                "Received proper request from client: {:}\n",
                &request_string
            ));
            self.send(&resp.result);
        } else {
            self.log(&format!(
                "Received invalid request string from client: {}\n response: {}\n",
                request_string, resp.result
            ));
        }
    }

    pub async fn run(&mut self) {
        self.log("Server has started\n");

        for line in stdin().lock().lines() {
            let line_string = line.unwrap();

            if line_string.is_empty() {
                break;
            }

            self.handle(&line_string).await;
        }
    }

    fn send(&self, message: &str) {
        println!("{}", message);
        self.log(&format!("Sent to client: {:}\n", &message));
    }

    fn log(&self, message: &str) {
        stderr().write_all(message.as_bytes()).unwrap();
    }
}

#[tokio::main]
pub async fn main() {
    Server::new().run().await;
}
