//! Testing connection with the client - initialize handshake and shutdown request.

use cargo_bsp::utils::tests::*;
use serde_json::to_string;

mod common;
use crate::common::{init_connection, spawn_server, spawn_server_with_proper_life_time, Client};

#[test]
fn proper_lifetime() {
    spawn_server_with_proper_life_time(|_| {});
}

#[test]
fn exit_notif_before_init() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    cl.send(&to_string(&test_exit_notif()).unwrap());
    assert_eq!(child.wait().unwrap().code(), Some(1));
}

#[test]
fn exit_notif_without_shutdown() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_connection(&mut cl);

    cl.send(&to_string(&test_exit_notif()).unwrap());
    assert_eq!(child.wait().unwrap().code(), Some(1));
}
