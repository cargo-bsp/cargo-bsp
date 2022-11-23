use nix::unistd::{dup2, fork, ForkResult, pipe};

use crate::client::Client;
use crate::server::Server;

mod bsp_types;
mod client;
mod server;
mod utils;

#[tokio::main]
async fn main() {
    let server_to_client = pipe().unwrap();
    let client_to_server = pipe().unwrap();

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child: _child, .. }) => {
                dup2(server_to_client.0, 0).unwrap();
                dup2(client_to_server.1, 1).unwrap();
                Client::new().run()
            }
            Ok(ForkResult::Child) => {
                dup2(client_to_server.0, 0).unwrap();
                dup2(server_to_client.1, 1).unwrap();
                Server::new().run().await;
            }
            Err(_) => println!("Fork failed"),
        }
    }
}
