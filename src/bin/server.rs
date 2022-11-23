use std::io::prelude::*;
use std::io::stdin;

use jsonrpsee_server::RpcModule;

use cargo_bsp::bsp_types::{InitializeBuildParams, InitializeBuildResult, MethodName};
use cargo_bsp::utils::{log, send};

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
            .register_method(InitializeBuildParams::get_method_name(), |params, _| {
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
                "Received invalid request string from client: {}\n response: {}\n",
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

pub fn main() {
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("Cannot read user input");
    let msg = format!("Server has received a message: {:?}\n", buf);
    stderr().write_all(msg.as_bytes()).expect("TODO: panic message");
    println!("Server finished");
}
