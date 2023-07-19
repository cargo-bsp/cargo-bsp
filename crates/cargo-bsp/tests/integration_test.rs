use std::process::{Child, Command, Stdio};

use assert_cmd::prelude::*;
use insta::{allow_duplicates, assert_snapshot};
use serde_json::to_string;

use cargo_bsp::utils::tests::{
    test_exit_notif, test_init_notif, test_init_params, test_init_req, test_shutdown_req,
};

use crate::common::client::Client;

mod common;

fn spawn_server() -> Child {
    Command::cargo_bin("server")
        .unwrap()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped()) // we don't want to see logs in tests
        .spawn()
        .unwrap()
}

fn init_connection(cl: &mut Client) {
    let test_id = 123;
    let init_params = test_init_params();

    cl.send(&to_string(&test_init_req(&init_params, test_id)).unwrap());

    allow_duplicates! {
        assert_snapshot!(cl.recv_resp(), @r###"{"jsonrpc":"2.0","id":123,"result":{"bspVersion":"2.0.0","capabilities":{"buildTargetChangedProvider":false,"canReload":true,"compileProvider":{"languageIds":[]},"dependencyModulesProvider":false,"dependencySourcesProvider":false,"inverseSourcesProvider":false,"jvmRunEnvironmentProvider":false,"jvmTestEnvironmentProvider":false,"outputPathsProvider":false,"resourcesProvider":false,"runProvider":{"languageIds":[]},"testProvider":{"languageIds":[]}},"displayName":"test","version":"0.0.1"}}"###);
    }

    cl.send(&to_string(&test_init_notif()).unwrap());
}

fn shutdown_connection(cl: &mut Client) {
    let test_id = 234;

    cl.send(&to_string(&test_shutdown_req(test_id)).unwrap());

    allow_duplicates! {
        assert_snapshot!(cl.recv_resp(), @r###"{"jsonrpc":"2.0","id":234,"result":null}"###);
    }

    cl.send(&to_string(&test_exit_notif()).unwrap());
}

fn spawn_server_with_proper_life_time(communication: fn(cl: &Client)) {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_connection(&mut cl);

    communication(&cl);

    shutdown_connection(&mut cl);
    assert_eq!(child.wait().unwrap().code(), Some(0));
}

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
#[ignore]
fn exit_notif_without_shutdown() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_connection(&mut cl);

    cl.send(&to_string(&test_exit_notif()).unwrap());
    assert_eq!(child.wait().unwrap().code(), Some(1));
}

mod feature_protocol_extension_integration_test {
    use super::spawn_server_with_proper_life_time;
    use cargo_toml_builder::types::Feature;
    use cargo_toml_builder::CargoToml;
    use std::fs::{remove_dir_all, File};
    use std::io::Write;
    use std::process::Command;

    const TEST_PROJECT_NAME: &str = "tmp_test_project";
    const TEST_PROJECT_AUTHOR: &str = "Test Author";
    const TEST_PROJECT_VERSION: &str = "0.0.1";

    const F: [&str; 6] = ["f0", "f1", "f2", "f3", "f4", "f5"];

    fn overwrite_cargo_toml_with_features(features_slice: &[(&str, &[&str])]) {
        let mut cargo_toml = CargoToml::builder();
        cargo_toml
            .name(TEST_PROJECT_NAME)
            .version(TEST_PROJECT_VERSION)
            .author(TEST_PROJECT_AUTHOR);

        for &feature_with_deps in features_slice {
            let mut f = Feature::new(feature_with_deps.0);
            for &dep in feature_with_deps.1 {
                f.feature(dep);
            }
            cargo_toml.feature(f.build());
        }

        let cargo_toml_string = cargo_toml.build().unwrap().to_string();
        print!("{}", cargo_toml_string);
        File::create("Cargo.toml")
            .unwrap()
            .write_all(cargo_toml_string.as_bytes())
            .unwrap();
    }

    fn create_mock_rust_project_with_features_and_set_it_as_current_dir(
        features_slice: &[(&str, &[&str])],
    ) {
        Command::new(toolchain::cargo())
            .args(["init", TEST_PROJECT_NAME])
            .output()
            .expect("Failed to create new temporary project for testing.");

        std::env::set_current_dir(TEST_PROJECT_NAME).unwrap();

        overwrite_cargo_toml_with_features(features_slice);
    }

    #[test]
    fn feature_enablement_test() {
        create_mock_rust_project_with_features_and_set_it_as_current_dir(&[
            (F[0], &[F[1]]),
            (F[1], &[F[3], F[4]]),
            (F[3], &[F[4]]),
            (F[4], &[]),
        ]);
        //TODO here add testing function
        spawn_server_with_proper_life_time(|_| {});

        std::env::set_current_dir("..").unwrap();
        remove_dir_all(TEST_PROJECT_NAME).unwrap();
    }
}
