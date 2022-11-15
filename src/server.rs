// You can run this with `cargo run --bin server

use std::io;
use std::io::prelude::*;
use std::io::stderr;
use jsonrpsee_types::{Id, Request, RequestSer};
use serde_json::Result;
use serde_json::value::RawValue;

use crate::bsp_types::{BuildClientCapabilities, InitializeBuildParams};

pub fn run_server() {
    stderr().write_all("Server has started\n".as_bytes()).unwrap();
    println!("Hello, it's me - server :)");


    let temp: InitializeBuildParams<()> = InitializeBuildParams {
        display_name: "test1".to_string(),
        version: "test2".to_string(),
        bsp_version: "test3".to_string(),
        root_uri: "test4".to_string(),
        capabilities: BuildClientCapabilities { language_ids: vec!["test5".to_string()] },
        data: None,
    };
    println!("{}", prepare_result(&temp));

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }

        let request = get_request(&line_string);
        match request {
            Ok(r) => {
                let msg = format!("wczytałem {:?}, jesteście super!\n", r);
                stderr().write_all(msg.as_bytes()).unwrap();
            }
            Err(_) => {
                let msg = format!("wczytałem {}, jesteście z siebie dumni?\n", line_string);
                stderr().write_all(msg.as_bytes()).unwrap();
            }
        }
    }
}

fn get_request(request_string: &str) -> Result<InitializeBuildParams> {
    let request: Request = serde_json::from_str(request_string)?;
    serde_json::from_str(request.params.map_or("", |x| x.get()))
}

fn prepare_result(struct_params: &InitializeBuildParams) -> String {
    let string_params = serde_json::to_string(struct_params).unwrap();
    let params = Some(RawValue::from_string(string_params).unwrap());

    let method = "test6";
    let id = Id::Number(0183);
    let result = RequestSer::borrowed(&id, &method, params.as_deref());

    serde_json::to_string(&result).unwrap()
}

// {"jsonrpc": "2.0", "method": "subtract", "params": { "displayName": "test1", "version": "test2", "bspVersion": "test3", "rootUri": "test4", "capabilities": { "languageIds": ["test5"] }}, "id": 3}
