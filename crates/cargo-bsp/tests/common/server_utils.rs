use crate::Client;
use assert_cmd::cargo::CommandCargoExt;
use cargo_bsp::utils::tests::{
    test_exit_notif, test_init_notif, test_init_params, test_init_req, test_shutdown_req,
};
use insta::{allow_duplicates, assert_snapshot};
use serde_json::to_string;
use std::process::{Child, Command, Stdio};

pub fn init_connection(cl: &mut Client) {
    let test_id = 123;
    let init_params = test_init_params();

    cl.send(&to_string(&test_init_req(&init_params, test_id)).unwrap());

    allow_duplicates! {
        assert_snapshot!(cl.recv_resp(), @r###"{"jsonrpc":"2.0","id":123,"result":{"bspVersion":"2.0.0","capabilities":{"buildTargetChangedProvider":false,"canReload":true,"cargoFeaturesProvider":true,"compileProvider":{"languageIds":[]},"dependencyModulesProvider":false,"dependencySourcesProvider":false,"inverseSourcesProvider":false,"jvmRunEnvironmentProvider":false,"jvmTestEnvironmentProvider":false,"outputPathsProvider":false,"resourcesProvider":false,"runProvider":{"languageIds":[]},"testProvider":{"languageIds":[]}},"displayName":"test","version":"0.0.1"}}"###);
    }

    cl.send(&to_string(&test_init_notif()).unwrap());
}

pub fn shutdown_connection(cl: &mut Client) {
    let test_id = 234;

    cl.send(&to_string(&test_shutdown_req(test_id)).unwrap());

    allow_duplicates! {
        assert_snapshot!(cl.recv_resp(), @r#"{"jsonrpc":"2.0","id":234,"result":null}"#);
    }

    cl.send(&to_string(&test_exit_notif()).unwrap());
}

pub fn spawn_server() -> Child {
    Command::cargo_bin("server")
        .unwrap()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // we don't want to see logs in tests
        .spawn()
        .unwrap()
}

pub fn spawn_server_with_proper_life_time<F>(communication: F)
where
    F: Fn(&mut Client),
{
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_connection(&mut cl);

    communication(&mut cl);

    shutdown_connection(&mut cl);
    assert_eq!(child.wait().unwrap().code(), Some(0));
}
