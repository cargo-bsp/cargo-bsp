use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};

use jsonrpsee_core::traits::ToRpcParams;
use jsonrpsee_core::Error;
use jsonrpsee_types::{Id, RequestSer};
use serde::Serialize;

use crate::bsp_types::{MethodName, RequestWrapper};

pub struct Client<'a> {
    buf_reader: BufReader<&'a mut ChildStdout>,
    child_stdin: ChildStdin,
    request_id: u64,
}

impl<'a> Client<'a> {
    pub fn new(child: &'a mut Child) -> Self {
        Self {
            buf_reader: BufReader::new(child.stdout.as_mut().unwrap()),
            child_stdin: child.stdin.take().unwrap(),
            request_id: 0,
        }
    }

    fn send(&mut self, msg: &str) {
        let msg_with_endline = msg.to_owned() + "\n";
        let no_bytes = match self.child_stdin.write(msg_with_endline.as_bytes()) {
            Ok(no_bytes) => Some(no_bytes),
            Err(_) => None,
        };

        println!("Client has send: {:?}", no_bytes);
    }

    pub fn get_response(&mut self) -> Option<String> {
        let mut buf = String::new();
        match self.buf_reader.read_line(&mut buf) {
            Ok(_) => Some(buf),
            Err(_) => None,
        }
    }

    pub fn send_request<T>(&mut self, request: T)
    where
        T: Send + Serialize + MethodName,
    {
        let request_string = self
            .create_request_string(RequestWrapper {
                request_params: request,
            })
            .unwrap();
        self.send(&request_string);
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
}
