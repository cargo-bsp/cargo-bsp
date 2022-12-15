use std::io::prelude::*;
use std::io::{stderr, stdin, Write};

use jsonrpsee_server::RpcModule;
use jsonrpsee_types::{Notification};
use serde_json::from_str;

use cargo_bsp::bsp_types::requests::{InitializeBuildParams, InitializeBuildResult};
use cargo_bsp::bsp_types::MethodName;

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
                        display_name: "patryk".to_string(),
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
                    // TODO send error message
                }
            }
            Err(e) => if let jsonrpsee_core::Error::ParseError(_) = e {
                let notification: Notification<()> = from_str(request_string).unwrap();
                self.log(format!("Got notifiaction {:?}\n", notification).as_str());
            }
        };
    }

    fn parse_headers(&mut self) -> usize {
        let mut header = String::new();
        let mut content_length = 0;

        stdin().read_line(&mut header).expect("Failed to read headers");
        self.log(format!("Got a header: {}\n", header).as_str());

        let val= header.split(": ").collect::<Vec<&str>>()[1];
        if val.as_bytes()[0].is_ascii_digit() {
            content_length = from_str::<usize>(val).unwrap();
            self.log(format!("Content length read: {}\n", content_length).as_str());
        };

        stdin().read_line(&mut header).expect("Failed to read newlines"); // reading newlines
        content_length
    }

    pub fn read_n_chars(&self, no_chars: usize) -> String {
        let mut buf = vec![0u8; no_chars];
        stdin().read_exact(&mut buf).expect("Failed to read the actual content");
        let content = String::from_utf8(buf).unwrap();
        content
    }

    pub async fn run(&mut self) {
        loop {
            let content_length = self.parse_headers();
            let content = self.read_n_chars(content_length);
            self.log(format!("The actual content of message: {}\n", content).as_str());
            self.handle(&content).await;
        }
    }

    fn add_headers(&self, message: &str) -> String {
        let mut prefix = String::from(format!("Content-Length: {}\r\n\r\n", message.len()));
        prefix.push_str(message);
        prefix
    }

    fn send(&self, message: &str) {
        // something crashes with the content length
        // client sends notifications event after timeout
        let response = self.add_headers(message);
        println!("{}", response);
        self.log(&format!("Sent to client: {:}\n", &response));
    }

    fn log(&self, message: &str) {
        stderr().write_all(message.as_bytes()).unwrap();
    }
}

#[tokio::main]
pub async fn main() {
    Server::new().run().await;
}
