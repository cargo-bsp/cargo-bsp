use std::process::{Child, Command, Stdio};

pub mod client;

pub fn spawn_server() -> Child {
    Command::new("cargo")
        .args(["run", "--release", "--bin", "server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}
