use serde::Serialize;
use serde_json::to_string;

use cargo_bsp::communication;
use cargo_bsp::test_utils::{
    test_exit_notif, test_init_notif, test_init_params, test_init_req, test_init_resp,
    test_init_result, test_shutdown_req, test_shutdown_resp,
};

use crate::common::client::Client;
use crate::common::spawn_server;

mod common;

#[derive(Serialize)]
struct JsonRpc {
    jsonrpc: &'static str,
    #[serde(flatten)]
    msg: communication::Message,
}

fn make_rpc_string(msg: communication::Message) -> String {
    to_string(&JsonRpc {
        jsonrpc: "2.0",
        msg,
    })
    .unwrap()
}

fn init_conn(cl: &mut Client) {
    let test_id = 123;
    let init_params = test_init_params();

    cl.send(&to_string(&test_init_req(&init_params, test_id)).unwrap());

    let init_resp = test_init_resp(&test_init_result(&init_params), test_id);
    assert_eq!(cl.recv_resp(), make_rpc_string(init_resp.into()));

    cl.send(&to_string(&test_init_notif()).unwrap());
}

fn shutdown_conn(cl: &mut Client) {
    let test_id = 234;

    cl.send(&to_string(&test_shutdown_req(test_id)).unwrap());

    assert_eq!(
        cl.recv_resp(),
        make_rpc_string(test_shutdown_resp(test_id).into())
    );

    cl.send(&to_string(&test_exit_notif()).unwrap());
}

#[test]
fn proper_lifetime() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_conn(&mut cl);
    shutdown_conn(&mut cl);
    assert_eq!(child.wait().unwrap().code(), Some(0));
}

#[test]
fn exit_notif_before_init() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    let exit_notif = test_exit_notif();
    cl.send(&to_string(&exit_notif).unwrap());
    assert_eq!(child.wait().unwrap().code(), Some(1));
}

#[test]
fn exit_notif_without_shutdown() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_conn(&mut cl);

    let exit_notif = test_exit_notif();
    cl.send(&to_string(&exit_notif).unwrap());
    assert_eq!(child.wait().unwrap().code(), Some(1));
}
