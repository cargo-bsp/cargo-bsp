use nix::unistd::{dup2, fork, ForkResult, pipe};

use crate::client::run_client;
use crate::server::run_server;

mod server;
mod bsp_types;
mod client;
mod utils;

fn main() {
    let server_to_client = pipe().unwrap();
    let client_to_server = pipe().unwrap();

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child: _child, .. }) => {
                dup2(server_to_client.0, 0).unwrap();
                dup2(client_to_server.1, 1).unwrap();
                run_client();
            }
            Ok(ForkResult::Child) => {
                dup2(client_to_server.0, 0).unwrap();
                dup2(server_to_client.1, 1).unwrap();
                run_server();
            }
            Err(_) => println!("Fork failed"),
        }
    }
}
