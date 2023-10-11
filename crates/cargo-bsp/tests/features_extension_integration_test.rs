//! Integration tests for Cargo extension of BSP. The test in this module changes working directory,
//! when adding extra tests remember to execute them sequentially.

use bsp4rs::bsp::StatusCode;
use bsp4rs::cargo::{
    CargoFeaturesState, CargoFeaturesStateResult, PackageFeatures, SetCargoFeatures,
    SetCargoFeaturesParams, SetCargoFeaturesResult,
};
use bsp4rs::rust::{Feature, FeatureDependencyGraph};
use bsp4rs::Request;
use bsp_server::Response;
use cargo_toml_builder::{types::Feature as TomlFeature, CargoToml};
use serde_json::to_string;
use std::collections::{BTreeMap, BTreeSet};
use std::env::{current_dir, set_current_dir};
use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use std::process::Command;
use tempfile::tempdir;

const TEST_REQUEST_ID: i32 = 123;
const TEST_PROJECT_NAME: &str = "tmp_test_project";

mod common;

use common::{spawn_server_with_proper_life_time, Client};

struct FeaturesState {
    enabled_features: BTreeSet<Feature>,
    available_features: FeatureDependencyGraph,
}

fn send_cargo_feature_state_request(cl: &mut Client) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: CargoFeaturesState::METHOD.into(),
        params: Default::default(),
    };
    cl.send(&to_string(&req).unwrap());
}

fn check_set_features_response_status_code(received_string: String) {
    let resp: Response = serde_json::from_str(&received_string).unwrap();
    let result = serde_json::from_value::<SetCargoFeaturesResult>(resp.result.unwrap()).unwrap();
    assert_eq!(result.status_code, StatusCode::Ok);
}

fn send_set_features_request_and_check_result(
    cl: &mut Client,
    package_id: &str,
    features_to_set: &BTreeSet<Feature>,
) {
    let req = bsp_server::Request {
        id: TEST_REQUEST_ID.into(),
        method: SetCargoFeatures::METHOD.into(),
        params: serde_json::to_value(SetCargoFeaturesParams {
            package_id: package_id.to_string(),
            features: features_to_set.clone(),
        })
        .unwrap(),
    };
    cl.send(&to_string(&req).unwrap());
    check_set_features_response_status_code(cl.recv_resp());
}

fn overwrite_cargo_toml_with_features(features: &FeatureDependencyGraph) {
    const TEST_PROJECT_AUTHOR: &str = "Test Author";
    const TEST_PROJECT_VERSION: &str = "0.0.1";

    let mut cargo_toml = CargoToml::builder();
    cargo_toml
        .name(TEST_PROJECT_NAME)
        .version(TEST_PROJECT_VERSION)
        .author(TEST_PROJECT_AUTHOR);

    for (f, dependencies) in features.deref() {
        // feature added to the Cargo.toml builder
        let mut toml_feature = TomlFeature::new(&f.0);
        dependencies.iter().for_each(|dep| {
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

fn create_mock_rust_project(features: &FeatureDependencyGraph) {
    Command::new(toolchain::cargo())
        .args(["init", ".", "--name", TEST_PROJECT_NAME])
        .output()
        .expect("Failed to create new temporary project for testing.");

    overwrite_cargo_toml_with_features(features);
}

fn run_test<F>(features: &FeatureDependencyGraph, test: F)
where
    F: Fn(&mut Client),
{
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
    expected_available: &FeatureDependencyGraph,
    expected_state: &BTreeSet<Feature>,
) {
    let features_state = features_state_from_response(package);
    assert_eq!(
        features_state.available_features,
        expected_available.clone()
    );
    assert_eq!(features_state.enabled_features, expected_state.clone());
}

fn request_state_and_check_it(
    cl: &mut Client,
    expected_available: &FeatureDependencyGraph,
    expected_state: &BTreeSet<Feature>,
) {
    send_cargo_feature_state_request(cl);
    let package = package_from_response(&cl.recv_resp());
    check_package_state(package, expected_available, expected_state);
}

fn feature(id: i8) -> Feature {
    Feature(format!("f{}", id))
}

fn feature_with_dependencies(id: i8, dependencies: Vec<i8>) -> (Feature, BTreeSet<Feature>) {
    (
        feature(id),
        dependencies.iter().map(|d| feature(*d)).collect(),
    )
}

#[test]
fn cargo_features_state() {
    let mut available_features: FeatureDependencyGraph = FeatureDependencyGraph::new(
        (0..4)
            .map(|id| feature_with_dependencies(id, vec![id + 1]))
            .collect::<BTreeMap<Feature, BTreeSet<Feature>>>(),
    );
    // Add an f4 on which f3 depends
    available_features.0.insert(feature(4), BTreeSet::new());

    let test_fn = |cl: &mut Client| {
        let mut state = BTreeSet::new();

        // First request, obtain the package id
        send_cargo_feature_state_request(cl);
        let resp = cl.recv_resp();
        let package = package_from_response(&resp);
        let package_id = package.package_id.clone();
        check_package_state(package, &available_features, &state);

        // Set state as  [f1, f2]
        state = BTreeSet::from([feature(1), feature(2)]);
        send_set_features_request_and_check_result(cl, &package_id, &state);
        // Enabled: [f1, f2]
        request_state_and_check_it(cl, &available_features, &state);

        // Set state as  [f1]
        state = BTreeSet::from([feature(1)]);
        send_set_features_request_and_check_result(cl, &package_id, &state);
        // Enabled: [f2]
        request_state_and_check_it(cl, &available_features, &state);

        // Set state as [f0, f2, f3]
        state = BTreeSet::from([feature(0), feature(3)]);
        send_set_features_request_and_check_result(cl, &package_id, &state);
        // Enabled: [f0, f2, f3]
        request_state_and_check_it(cl, &available_features, &state);
    };

    run_test(&available_features, test_fn);
}
