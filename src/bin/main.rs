use std::process::{Child, Command, Stdio};

use cargo_bsp::bsp_types::InitializeBuildParams;
use cargo_bsp::client::Client;

fn test_first_query(mut cl: Client) {
    cl.send_request(InitializeBuildParams::default());
    println!("Sent a query, waiting for a response...");
    println!("Client got the response: {:?}", cl.get_response().unwrap());
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
