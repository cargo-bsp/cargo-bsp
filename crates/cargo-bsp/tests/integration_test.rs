use std::process::{Child, Command, Stdio};

use assert_cmd::prelude::*;
use insta::{allow_duplicates, assert_snapshot};
use serde_json::to_string;

use cargo_bsp::utils::tests::*;

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

fn spawn_server_with_proper_life_time(communication: fn(cl: &mut Client)) {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_connection(&mut cl);

    communication(&mut cl);

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
fn exit_notif_without_shutdown() {
    let mut child = spawn_server();
    let mut cl = Client::new(&mut child);

    init_connection(&mut cl);

    cl.send(&to_string(&test_exit_notif()).unwrap());
    assert_eq!(child.wait().unwrap().code(), Some(1));
}

mod feature_protocol_extension_integration_test {
    use super::*;
    use bsp_server::Response;
    use bsp_types::requests::{
        CargoFeaturesState, CargoFeaturesStateResult, DisableCargoFeatures,
        DisableCargoFeaturesParams, EnableCargoFeatures, EnableCargoFeaturesParams, Feature,
        PackageFeatures, Request,
    };
    use cargo_toml_builder::{types::Feature as TomlFeature, CargoToml};
    use regex::Regex;
    use serial_test::serial;
    use std::collections::BTreeSet;
    use std::fs::{remove_dir_all, File};
    use std::io::Write;
    use std::process::Command;

    const TEST_REQUEST_ID: i32 = 123;
    const TEST_PROJECT_NAME: &str = "tmp_test_project";

    // Features test cases
    const F: [&str; 6] = ["f0", "f1", "f2", "f3", "f4", "f5"];

    fn test_cargo_feature_state_request() -> bsp_server::Request {
        bsp_server::Request {
            id: TEST_REQUEST_ID.into(),
            method: CargoFeaturesState::METHOD.into(),
            params: Default::default(),
        }
    }

    fn test_enable_feature_request(
        package_id: &str,
        features_to_enable: &[&str],
    ) -> bsp_server::Request {
        bsp_server::Request {
            id: TEST_REQUEST_ID.into(),
            method: EnableCargoFeatures::METHOD.into(),
            params: serde_json::to_value(EnableCargoFeaturesParams {
                package_id: package_id.to_string(),
                features: features_to_enable.iter().map(|&s| s.into()).collect(),
            })
            .unwrap(),
        }
    }

    fn _test_disable_feature_request(
        package_id: &str,
        features_to_disable: &[&str],
    ) -> bsp_server::Request {
        bsp_server::Request {
            id: TEST_REQUEST_ID.into(),
            method: DisableCargoFeatures::METHOD.into(),
            params: serde_json::to_value(DisableCargoFeaturesParams {
                package_id: package_id.to_string(),
                features: features_to_disable.iter().map(|&s| s.into()).collect(),
            })
            .unwrap(),
        }
    }

    fn overwrite_cargo_toml_with_features(features_slice: &[(&str, &[&str])]) {
        const TEST_PROJECT_AUTHOR: &str = "Test Author";
        const TEST_PROJECT_VERSION: &str = "0.0.1";

        let mut cargo_toml = CargoToml::builder();
        cargo_toml
            .name(TEST_PROJECT_NAME)
            .version(TEST_PROJECT_VERSION)
            .author(TEST_PROJECT_AUTHOR);

        for &feature_with_deps in features_slice {
            let mut f = TomlFeature::new(feature_with_deps.0);
            for &dep in feature_with_deps.1 {
                f.feature(dep);
            }
            cargo_toml.feature(f.build());
        }

        let cargo_toml_string = cargo_toml.build().unwrap().to_string();
        File::create("Cargo.toml")
            .unwrap()
            .write_all(cargo_toml_string.as_bytes())
            .unwrap();
    }

    // Function that creates new temporary project with Cargo.toml with features
    // and sets it as current directory. The newly created directory has to be deleted.
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

    fn run_test(features_slice: &[(&str, &[&str])], test: fn(&mut Client)) {
        create_mock_rust_project_with_features_and_set_it_as_current_dir(features_slice);

        spawn_server_with_proper_life_time(test);

        // Do the cleaning
        std::env::set_current_dir("..").unwrap();
        // remove_dir_all(TEST_PROJECT_NAME).unwrap();
    }
    // Returns pair of available and enabled features from CargoFeatureState response
    fn features_state_from_response(resp: &str) -> (BTreeSet<Feature>, BTreeSet<Feature>) {
        let resp: Response = serde_json::from_str(resp).unwrap();
        let mut current_state: CargoFeaturesStateResult =
            serde_json::from_value(resp.result.unwrap()).unwrap();
        assert_eq!(current_state.packages_features.len(), 1);
        let package = current_state.packages_features.pop().unwrap();
        (package.available_features, package.enabled_features)
    }

    fn check_state(resp: &str, expected_available: &[&str], expected_enabled: &[&str]) {
        let (available_features, enabled_features) = features_state_from_response(resp);
        assert_eq!(
            available_features,
            create_feature_set_from_slices(expected_available)
        );
        assert_eq!(
            enabled_features,
            create_feature_set_from_slices(expected_enabled)
        );
    }

    #[test]
    #[serial]
    fn cargo_features_state() {
        let test_fn = |cl: &mut Client| {
            let expected_available: &[&str] = &[F[0], F[1], F[3], F[4]];
            let mut expected_enabled: Vec<&str> = vec![];
            cl.send(&to_string(&test_cargo_feature_state_request()).unwrap());
            check_state(&cl.recv_resp(), expected_available, &expected_enabled);

            let toggle_features: &[&str] = &[F[1], F[2]];
            expected_enabled.append(&mut toggle_features.to_vec());
            cl.send(&to_string(&test_enable_feature_request(
                "tmp_test_project 0.0.1 (path+file:///home/tomek/studia/ZPP/cargo-projects/cargo-bsp-dev/crates/cargo-bsp/tmp_test_project)",
                toggle_features)).unwrap());

            cl.send(&to_string(&test_cargo_feature_state_request()).unwrap());
            check_state(&cl.recv_resp(), expected_available, &expected_enabled);
        };

        run_test(
            &[
                (F[0], &[F[1]]),
                (F[1], &[F[3], F[4]]),
                (F[3], &[F[4]]),
                (F[4], &[]),
            ],
            test_fn,
        );
    }

    // #[test]
    // #[serial]
    // fn enable_cargo_features() {
    //     let test_fn = |_cl: &mut Client| {
    //         // cl.send(
    //         //     &to_string(&test_enable_feature_request(
    //         //         "tmp_test_project 0.0.1 ",
    //         //         &[F[2]],
    //         //     ))
    //         //     .unwrap(),
    //         // );
    //         // let resp = cl.recv_resp();
    //         // assert_snapshot!(remove_absolute_paths_from_response(&resp), @"");
    //         //
    //         // let current_state = current_state_from_response(&resp);
    //         // let (available_features, enabled_features) = features_state(&current_state.packages_features, "tmp_test_project 0.0.1 ".into());
    //         // assert_eq!(available_features, create_feature_set_from_slices(&[F[0], F[1], F[2], F[3], F[4]]));
    //         // //todo make it without the snapshots
    //     };
    //
    //     run_test(
    //         &[
    //             (F[0], &[F[1]]),
    //             (F[1], &[F[3], F[4]]),
    //             (F[3], &[F[4]]),
    //             (F[4], &[]),
    //         ],
    //         test_fn,
    //     );
    // }

    // _______________________________________________________________________________

    fn _remove_absolute_paths_from_response(resp: &str) -> String {
        // Remove absolute path from package ids
        const REG_PACKAGE_ID: &str = r#"\(path\+[^)]*\)"#;
        let mut re = Regex::new(REG_PACKAGE_ID).unwrap();
        let resp = re.replace(resp, "").to_string();

        // Remove absolute path from build target ids
        const REG_BUILD_TARGET: &str = r#"targetId:[\w\/\\-]*[\/\\]"#;
        re = Regex::new(REG_BUILD_TARGET).unwrap();
        re.replace(&resp, "targetId:").to_string()
    }

    fn _current_state_from_response(resp: &str) -> CargoFeaturesStateResult {
        let resp: Response = serde_json::from_str(resp).unwrap();
        serde_json::from_value(resp.result.unwrap()).unwrap()
    }

    // Returns pair of available and enabled features for packageId
    fn _features_state(
        packages_features: &[PackageFeatures],
        package_id: String,
    ) -> (BTreeSet<Feature>, BTreeSet<Feature>) {
        packages_features
            .iter()
            .find(|&p| p.package_id == package_id)
            .map(|p| (p.available_features.clone(), p.enabled_features.clone()))
            .unwrap()
    }
}
