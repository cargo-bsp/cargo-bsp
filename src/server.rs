// You can run this with `cargo run --bin server

// use std::borrow::Borrow;
use std::io;
use std::io::prelude::*;

// use beef::Cow;
// use jsonrpsee_types::Id::Null;
use jsonrpsee_types::Request;
use serde_json::Result;

mod bsp_types;
use crate::bsp_types::InitializeBuildParams;

// use std::ops::Deref;

// use serde_json::value::RawValue;

pub fn main() {
    println!("Hello, it's me - server :)");

    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line_string = line.unwrap();

        if line_string.is_empty() {
            break;
        }

        let request = get_request(&line_string);
        match request {
            Ok(r) => println!("wczytałem {:?}, jesteście super!", r),
            Err(_) => println!("wczytałem {}, jesteście z siebie dumni?", line_string),
        }
    }
}

fn get_request(request_string: &str) -> Result<InitializeBuildParams> {
    let request: Request = serde_json::from_str(request_string)?;
    serde_json::from_str(request.params.map_or("", |x| x.get()))
}

// {"jsonrpc": "2.0", "method": "subtract", "params": { "displayName": "test1", "version": "test2", "bspVersion": "test3", "rootUri": "test4", "capabilities": { "languageIds": ["test5"] }}, "id": 3}

// let temp: InitializeBuildParams<()> = InitializeBuildParams {
//     display_name: "test1".to_string(),
//     version: "test2".to_string(),
//     bsp_version: "test3".to_string(),
//     root_uri: "test4".to_string(),
//     capabilities: BuildClientCapabilities { language_ids: vec!["test5".to_string()] },
//     data: None
// };
// let temp_string = serde_json::to_string(&temp).unwrap();
// let temp3 = RawValue::from_string(temp_string).unwrap().borrow();
// let temp2 = Request::new(Cow::borrowed("test6"), Some(temp3), Null);
