use std::io::prelude::*;
use std::io::{stderr, stdin, Write};
use std::str::{from_utf8};

use jsonrpsee_server::RpcModule;
use jsonrpsee_types::{Notification, NotificationSer};
use serde_json::from_str;

use cargo_bsp::bsp_types::{InitializeBuildParams, InitializeBuildResult, MethodName};
use cargo_bsp::bsp_types::notifications::InitializedBuildParams;

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
                    .map::<InitializeBuildResult, _>(|_| InitializeBuildResult {
                        display_name : "patryk".to_string(),
                        version: "0.0.1".to_string(),
                        bsp_version: "2.0.0".to_string(),
                        capabilities: Default::default(),
                        data: None,
                    })
                    .map_err(Into::into)
            })
            .unwrap();
        module
    }

    async fn handle(&self, request_string: &str) {
        self.log(format!("Request string {}\n", request_string).as_str());
        match self.module.raw_json_request(request_string).await {
            Ok((resp, _)) => {
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
            Err(e) => if let jsonrpsee_core::Error::ParseError(_) = e {
                let notification: Notification<()> = from_str(request_string).unwrap();
                self.log(format!("Got notifiaction {:?}\n", notification).as_str());
            }
        };
    }

    pub async fn run(&mut self) {
        loop {
            let mut len_string = String::new();
            stdin().read_line(&mut len_string).expect("Failed to read content length");
            let mut content_length = 20;
            self.log(format!("Got the first line: {}\n", len_string).as_str());

            let xd = len_string.split_whitespace().last().unwrap_or("2137");
            if xd.as_bytes()[0].is_ascii_digit() {
                content_length = from_str::<usize>(xd).unwrap();
                self.log(format!("Content length read: {}\n", content_length).as_str());
            }

            let mut buf = vec![0u8; content_length + 2];
            stdin().read_exact(&mut buf).expect("Failed to read the actual request");
            let actual_req = from_utf8(&buf).unwrap();
            self.log(format!("Actual content of message: {}\n", actual_req).as_str());

            self.handle(&actual_req).await;
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
