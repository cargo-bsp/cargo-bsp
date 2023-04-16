//! Code in this file concerns the discovery of unit tests in a project.
//!
//! Current implementation is based on the mapping Cargo metadata targets to BSP build targets.
//! Meaning that unit tests are not considered as a separate build targets.
//!
//! Unit tests discovery is possible using
//!
//! *`cargo +nightly test -- --list --format json -Zunstable-options`*
//!
//! however, this requires compilation of whole project, which is not desirable -
//! it is time consuming and generates additional files.
//!
//! Command mentioned above discovers all tests, also *integration tests*,
//! which are treated as a build targets by 'cargo metadata', so they need to be skipped or merged
//! with the ones discovered in [ProjectWorkspace::new()](crate::project_model::workspace::ProjectWorkspace::new).
//!
//! Moreover, this approach gives us the information whether tests are ignored or not.
//! Not achieved with 'cargo metadata'.
//!
//! **Alternative next steps:** Analyze how cargo discovers tests and compile only
//! to the moment when unit tests can be obtained.

use crate::bsp_types::RustBuildTargetData::Rust;
use crate::bsp_types::{
    BuildTarget, BuildTargetCapabilities, BuildTargetIdentifier, RustBuildTarget, RUST_ID,
};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::Edition;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

/// Struct for deserialization of test JSON from
/// 'cargo +nightly test -- --list --format json -Zunstable-options' output
///
/// Deserialization truncates discovered test JSON as below
///
/// ```json
/// {
///    "type": "test",
///    "event": "discovered",
///    "name": "bsp_types::notifications::exit_build::tests::exit_build_method",
///    "ignore": false,
///    "ignore_message": "",
///    "source_path": "src/bsp_types/notifications/exit_build.rs",
///    "start_line": 18,
///    "start_col": 8,
///    "end_line": 18,
///    "end_col": 25
/// }
///
/// CargoTestTarget {
///     name: "bsp_types::notifications::exit_build::tests::exit_build_method",
///     ignore: false,
///     ignore_message: "",
///     source_path: "src/bsp_types/notifications/exit_build.rs",
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
struct CargoTestTarget {
    /// name of the test
    pub name: String,
    /// path to the file, where test is defined
    pub source_path: Utf8PathBuf,
    /// whether this test is marked with #[ignore] and should not be compiled
    ignore: bool,
    /// displayed message for ignored test
    ignore_message: String,
}

impl From<&CargoTestTarget> for BuildTarget {
    /// **Unfinished** implementation of mapping CargoTestTarget to BuildTarget
    fn from(test_target: &CargoTestTarget) -> Self {
        let rust_specific_data = Rust(RustBuildTarget {
            edition: Edition::E2021,
            required_features: vec![],
        });

        let mut base_directory = test_target.source_path.clone();
        // we assume that cargo test returns valid path to file, which additionally has a parent
        base_directory.pop();

        BuildTarget {
            id: BuildTargetIdentifier {
                uri: format!("{}:{}", test_target.source_path, test_target.name),
            },
            display_name: Some(test_target.name.clone()),
            base_directory: Some(format!("file://{}", base_directory)),
            tags: vec![],
            capabilities: BuildTargetCapabilities::default(),
            language_ids: vec![RUST_ID.to_string()],
            dependencies: vec![],
            data: Some(rust_specific_data),
        }
    }
}

/// Command call and deserialization
fn get_test_targets_from_cargo_test() -> Vec<CargoTestTarget> {
    let mut command = Command::new("cargo")
        .args([
            "+nightly",
            "test",
            "--",
            "--list",
            "--format",
            "json",
            "-Zunstable-options",
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut tests_targets: Vec<CargoTestTarget> = vec![];

    let reader = BufReader::new(command.stdout.take().unwrap());
    for line in reader.lines().map(|l| l.unwrap()) {
        let deserialized: CargoTestTarget = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        tests_targets.push(deserialized);
    }

    tests_targets
}

/// **Unfinished** - mapping of *test_targets* is not implemented yet.
#[allow(dead_code)]
pub fn get_unit_tests_build_targets() -> Vec<BuildTarget> {
    let test_targets = get_test_targets_from_cargo_test();
    test_targets
        .iter()
        .map(|test_target| test_target.into())
        .collect()
}
