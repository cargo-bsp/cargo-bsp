use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::io::{Write, BufReader, BufRead};
use crate::bsp_types::{BuildClientCapabilities, InitializeBuildParams, RequestRPC};

mod bsp_types;

fn example_client_initialize_query() -> InitializeBuildParams {
    InitializeBuildParams {
        display_name: "rust-bsp-client".to_string(),
        version: "0.1.0".to_string(),
        bsp_version: "2.0.0-M5".to_string(),
        root_uri: "file:///home/jan/pawel/ii/Projects/rust-bsp-client".to_string(),
        capabilities: BuildClientCapabilities {
            language_ids: vec!["rust".to_string()],
        },
        data: None,
    }
}

struct Client<'a> {
    buf_reader: BufReader<&'a mut ChildStdout>,
    child_stdin: ChildStdin,
}

impl<'a> Client<'a> {
    fn new(child: &'a mut Child) -> Self {
        Self {
            buf_reader: BufReader::new(child.stdout.as_mut().unwrap()),
            child_stdin: child.stdin.take().unwrap(),
        }
    }

    fn send(&mut self, msg: &str) -> Option<usize> {
        match self.child_stdin.write(msg.as_bytes()) {
            Ok(no_bytes) => Some(no_bytes),
            Err(_) => None
        }
    }

    fn get_response(&mut self) -> Option<String> {
        let mut buf = String::new();
        match self.buf_reader.read_line(&mut buf) {
            Ok(_) => Some(buf),
            Err(_) => None
        }
    }
}

fn test_first_query(mut cl: Client) {
    let first_query = example_client_initialize_query().parse_to_string() + "\n";
    cl.send(&first_query);
    println!("Sent the first query, waiting for a response...");
    println!("Response: {:?}", cl.get_response().unwrap());
}

fn spawn_server() -> Child {
    Command::new("cargo")
        .args(["run", "--bin", "server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

fn main() {
    let mut child = spawn_server();

    let cl = Client::new(&mut child);
    println!("Created a client");

    test_first_query(cl);
}
