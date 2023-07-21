//! Integration tests for Cargo extension of BSP. The test in this module switches te environment,
//! when adding extra tests remember to execute them sequentially.

use bsp_server::Response;
use bsp_types::requests::cargo_extension::*;
use bsp_types::requests::Request;
use cargo_bsp::utils::tests::create_feature_set_from_slices;
use cargo_toml_builder::{types::Feature as TomlFeature, CargoToml};
use serde_json::to_string;
use std::collections::BTreeSet;
use std::env::{current_dir, set_current_dir};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

const TEST_REQUEST_ID: i32 = 123;
const TEST_PROJECT_NAME: &str = "tmp_test_project";

mod common;
use common::{spawn_server_with_proper_life_time, Client};

const F: [&str; 6] = ["f0", "f1", "f2", "f3", "f4", "f5"];

struct FeaturesState {
    enabled_features: BTreeSet<Feature>,
    available_features: BTreeSet<Feature>,
}

struct FeatureSlice {
    name: &'static str,
    dependencies: &'static [&'static str],
}

impl FeatureSlice {
    pub fn new(name: &'static str, dependencies: &'static [&str]) -> Self {
        FeatureSlice { name, dependencies }
    }
}

fn send_cargo_feature_state_request(cl: &mut Client) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: CargoFeaturesState::METHOD.into(),
        params: Default::default(),
    };
    cl.send(&to_string(&req).unwrap());
}

fn send_enable_features_request(cl: &mut Client, package_id: &str, features_to_enable: &[&str]) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: EnableCargoFeatures::METHOD.into(),
        params: serde_json::to_value(EnableCargoFeaturesParams {
            package_id: package_id.to_string(),
            features: features_to_enable.iter().map(|&s| s.into()).collect(),
        })
        .unwrap(),
    };
    cl.send(&to_string(&req).unwrap());
}

fn send_disable_features_request(cl: &mut Client, package_id: &str, features_to_disable: &[&str]) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: DisableCargoFeatures::METHOD.into(),
        params: serde_json::to_value(DisableCargoFeaturesParams {
            package_id: package_id.to_string(),
            features: features_to_disable.iter().map(|&s| s.into()).collect(),
        })
        .unwrap(),
    };
    cl.send(&to_string(&req).unwrap());
}

fn overwrite_cargo_toml_with_features(features_slice: &[FeatureSlice]) {
    const TEST_PROJECT_AUTHOR: &str = "Test Author";
    const TEST_PROJECT_VERSION: &str = "0.0.1";

    let mut cargo_toml = CargoToml::builder();
    cargo_toml
        .name(TEST_PROJECT_NAME)
        .version(TEST_PROJECT_VERSION)
        .author(TEST_PROJECT_AUTHOR);

    for feature_slice in features_slice {
        let mut f = TomlFeature::new(feature_slice.name);
        for &dep in feature_slice.dependencies {
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

fn create_mock_rust_project(features_slice: &[FeatureSlice]) {
    Command::new(toolchain::cargo())
        .args(["init", ".", "--name", TEST_PROJECT_NAME])
        .output()
        .expect("Failed to create new temporary project for testing.");

    overwrite_cargo_toml_with_features(features_slice);
}

fn run_test(features_slice: &[FeatureSlice], test: fn(&mut Client)) {
    let starting_path = current_dir().unwrap();
    let tmp_dir = tempdir().unwrap();
    set_current_dir(tmp_dir.path()).unwrap();
    create_mock_rust_project(features_slice);

    spawn_server_with_proper_life_time(test);
    set_current_dir(starting_path).unwrap();
}

fn package_from_response(resp: &str) -> PackageFeatures {
    let resp: Response = serde_json::from_str(resp).unwrap();
    let mut current_state: CargoFeaturesStateResult =
        serde_json::from_value(resp.result.unwrap()).unwrap();
    assert_eq!(current_state.packages_features.len(), 1);
    current_state.packages_features.pop().unwrap()
}

fn package_id_from_response(resp: &str) -> String {
    package_from_response(resp).package_id
}

// Returns pair of available and enabled features from CargoFeatureState response
fn features_state_from_response(resp: &str) -> FeaturesState {
    let package = package_from_response(resp);
    FeaturesState {
        available_features: package.available_features,
        enabled_features: package.enabled_features,
    }
}

fn check_state(resp: &str, expected_available: &[&str], expected_enabled: &[&str]) {
    let features_state = features_state_from_response(resp);
    assert_eq!(
        features_state.available_features,
        create_feature_set_from_slices(expected_available)
    );
    assert_eq!(
        features_state.enabled_features,
        create_feature_set_from_slices(expected_enabled)
    );
}

#[test]
fn cargo_features_state() {
    let test_fn = |cl: &mut Client| {
        let expected_available: &[&str] = &[F[0], F[1], F[2], F[3]];
        let mut expected_enabled: Vec<&str> = vec![];
        let mut toggle_features: Vec<&str> = vec![];

        send_cargo_feature_state_request(cl);
        let resp = cl.recv_resp();
        let package_id = package_id_from_response(&resp);
        check_state(&resp, expected_available, &expected_enabled);

        // Enable f1, f2
        toggle_features.extend(&[F[1], F[2]]);
        expected_enabled.extend(&toggle_features);
        send_enable_features_request(cl, &package_id, &toggle_features);
        cl.recv_resp();
        // Enabled: [f1, f2]
        send_cargo_feature_state_request(cl);
        check_state(&cl.recv_resp(), expected_available, &expected_enabled);

        // Disable f1
        toggle_features.clear();
        toggle_features.extend(&[F[1]]);
        expected_enabled.retain(|&f| !toggle_features.contains(&f));
        send_disable_features_request(cl, &package_id, &toggle_features);
        cl.recv_resp();
        // Enabled: [f2]
        send_cargo_feature_state_request(cl);
        check_state(&cl.recv_resp(), expected_available, &expected_enabled);

        // Enable f0, f3
        toggle_features.clear();
        toggle_features.extend(&[F[0], F[3]]);
        expected_enabled.extend(&toggle_features); // Enabled: [f0, f2, f3]
        send_enable_features_request(cl, &package_id, &toggle_features);
        cl.recv_resp();
        // Enabled: [f0, f2, f3]
        send_cargo_feature_state_request(cl);
        check_state(&cl.recv_resp(), expected_available, &expected_enabled);
    };

    run_test(
        &[
            FeatureSlice::new(F[0], &[F[1]]),
            FeatureSlice::new(F[1], &[F[3], F[2]]),
            FeatureSlice::new(F[2], &[F[3]]),
            FeatureSlice::new(F[2], &[F[3]]),
            FeatureSlice::new(F[3], &[]),
        ],
        test_fn,
    );
}
