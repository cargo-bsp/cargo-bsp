//! Integration tests for Cargo extension of BSP. The test in this module changes working directory,
//! when adding extra tests remember to execute them sequentially.

use bsp_server::Response;
use bsp_types::requests::cargo_extension::*;
use bsp_types::requests::Request;
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

struct FeaturesState {
    enabled_features: BTreeSet<Feature>,
    available_features: BTreeSet<Feature>,
}

struct FeatureWithDependencies {
    name: Feature,
    dependencies: Vec<Feature>,
}

impl FeatureWithDependencies {
    pub fn new(name: Feature, dependencies: &[Feature]) -> Self {
        FeatureWithDependencies {
            name,
            dependencies: dependencies.to_vec(),
        }
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

fn send_enable_features_request(
    cl: &mut Client,
    package_id: &str,
    features_to_enable: BTreeSet<Feature>,
) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: EnableCargoFeatures::METHOD.into(),
        params: serde_json::to_value(EnableCargoFeaturesParams {
            package_id: package_id.to_string(),
            features: features_to_enable,
        })
        .unwrap(),
    };
    cl.send(&to_string(&req).unwrap());
}

fn send_disable_features_request(
    cl: &mut Client,
    package_id: &str,
    features_to_disable: BTreeSet<Feature>,
) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: DisableCargoFeatures::METHOD.into(),
        params: serde_json::to_value(DisableCargoFeaturesParams {
            package_id: package_id.to_string(),
            features: features_to_disable,
        })
        .unwrap(),
    };
    cl.send(&to_string(&req).unwrap());
}

fn overwrite_cargo_toml_with_features(features: &[FeatureWithDependencies]) {
    const TEST_PROJECT_AUTHOR: &str = "Test Author";
    const TEST_PROJECT_VERSION: &str = "0.0.1";

    let mut cargo_toml = CargoToml::builder();
    cargo_toml
        .name(TEST_PROJECT_NAME)
        .version(TEST_PROJECT_VERSION)
        .author(TEST_PROJECT_AUTHOR);

    for f in features {
        // feature added to the Cargo.toml builder
        let mut toml_feature = TomlFeature::new(&f.name.0);
        f.dependencies.iter().for_each(|dep| {
            toml_feature.feature(&dep.0);
        });
        cargo_toml.feature(toml_feature.build());
    }

    let cargo_toml_string = cargo_toml.build().unwrap().to_string();
    File::create("Cargo.toml")
        .unwrap()
        .write_all(cargo_toml_string.as_bytes())
        .unwrap();
}

fn create_mock_rust_project(features: &[FeatureWithDependencies]) {
    Command::new(toolchain::cargo())
        .args(["init", ".", "--name", TEST_PROJECT_NAME])
        .output()
        .expect("Failed to create new temporary project for testing.");

    overwrite_cargo_toml_with_features(features);
}

fn run_test(features: &[FeatureWithDependencies], test: fn(&mut Client)) {
    let starting_path = current_dir().unwrap();
    let tmp_dir = tempdir().unwrap();
    set_current_dir(tmp_dir.path()).unwrap();
    create_mock_rust_project(features);

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

fn features_state_from_response(package: PackageFeatures) -> FeaturesState {
    FeaturesState {
        available_features: package.available_features,
        enabled_features: package.enabled_features,
    }
}

fn check_package_state(
    package: PackageFeatures,
    expected_available: &BTreeSet<Feature>,
    expected_enabled: &BTreeSet<Feature>,
) {
    let features_state = features_state_from_response(package);
    assert_eq!(
        features_state.available_features,
        expected_available.clone()
    );
    assert_eq!(features_state.enabled_features, expected_enabled.clone());
}

fn request_state_and_check_it(
    cl: &mut Client,
    expected_available: &BTreeSet<Feature>,
    expected_enabled: &BTreeSet<Feature>,
) {
    send_cargo_feature_state_request(cl);
    let package = package_from_response(&cl.recv_resp());
    check_package_state(package, expected_available, expected_enabled);
}

fn feature(id: i8) -> Feature {
    Feature(format!("f{}", id))
}

#[test]
fn cargo_features_state() {
    let test_fn = |cl: &mut Client| {
        let expected_available: BTreeSet<Feature> = (0..4).map(feature).collect();
        let mut expected_enabled = BTreeSet::new();
        let mut toggle_features;

        send_cargo_feature_state_request(cl);
        let resp = cl.recv_resp();
        let package = package_from_response(&resp);
        let package_id = package.package_id.clone();
        check_package_state(package, &expected_available, &expected_enabled);

        // Enable f1, f2
        toggle_features = BTreeSet::from([feature(1), feature(2)]);
        expected_enabled.extend(toggle_features.clone());
        send_enable_features_request(cl, &package_id, toggle_features);
        cl.recv_resp();
        // Enabled: [f1, f2]
        request_state_and_check_it(cl, &expected_available, &expected_enabled);

        // Disable f1
        toggle_features = BTreeSet::from([feature(1)]);
        expected_enabled.retain(|f| !toggle_features.contains(f));
        send_disable_features_request(cl, &package_id, toggle_features);
        cl.recv_resp();
        // Enabled: [f2]
        request_state_and_check_it(cl, &expected_available, &expected_enabled);

        // Enable f0, f3
        toggle_features = BTreeSet::from([feature(0), feature(3)]);
        expected_enabled.extend(toggle_features.clone()); // Enabled: [f0, f2, f3]
        send_enable_features_request(cl, &package_id, toggle_features);
        cl.recv_resp();
        // Enabled: [f0, f2, f3]
        request_state_and_check_it(cl, &expected_available, &expected_enabled);
    };

    run_test(
        &[
            FeatureWithDependencies::new(feature(0), &[feature(1)]),
            FeatureWithDependencies::new(feature(1), &[feature(3), feature(2)]),
            FeatureWithDependencies::new(feature(2), &[feature(3)]),
            FeatureWithDependencies::new(feature(2), &[feature(3)]),
            FeatureWithDependencies::new(feature(3), &[]),
        ],
        test_fn,
    );
}
