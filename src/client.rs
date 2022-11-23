use std::io::stdin;

use jsonrpsee_core::Error;
use jsonrpsee_core::traits::ToRpcParams;
use jsonrpsee_types::{Id, RequestSer};
use serde::Serialize;

use crate::bsp_types::{InitializeBuildParams, MethodName, RequestWrapper};
use crate::utils::{log, send};

pub struct Client {
    request_id: u64,
}

impl Client {
    pub fn new() -> Self {
        Self { request_id: 0 }
    }

    fn create_request_string<T>(&mut self, request: RequestWrapper<T>) -> Result<String, Error>
    where
        T: Send + Serialize + MethodName,
    {
        let id = Id::Number(self.request_id);
        self.request_id += 1;
        let method = T::get_method_name();
        let params = request.to_rpc_params()?;

        let request = RequestSer::borrowed(&id, &method, params.as_deref());
        serde_json::to_string(&request).map_err(Error::ParseError)
    }

    pub fn run(&mut self) {
        log("Client started\n");

        let request_string = self
            .create_request_string(RequestWrapper {
                request_params: InitializeBuildParams::default(),
            })
            .unwrap();

        send(&request_string);
        let mut line = String::new();
        stdin().read_line(&mut line).unwrap();
        log(&format!("Received message from server: {}\n", line));
    }
}
