//! CreateCommand trait implementation for the Compile/Run/Test/RustWorkspaceParams.
//! The trait allows creating commands regardless if it is the compile, run, test or rust_workspace request.
//!
//! The created commands are: `cargo build`, `cargo run`, `cargo test` and `cargo check` to compile,
//! run and test the project.
//!
//! The requested commands have additional flags:
//!
//! `--message-format=json` for all commands. This flag formats information to JSON and
//! provides [additional information about build](https://doc.rust-lang.org/cargo/reference/external-tools.html)
//!
//! `--show-output -Z unstable-options --format=json` for `cargo test`
//! (only with `+nightly`). These flags format information about the tests to JSON and
//! allows additional information, such as when each single tests started and finished,
//! their stdout and stderr
//!
//! `--workspace --all-targets -Z unstable-options --keep-going` for `cargo check`.
//! `--all-targets` is needed to compile:
//! - build scripts even if a crate doesn't contain library or binary targets,
//! - dev dependencies during build script evaluation
//! `--keep-going` is needed to compile as many proc macro artifacts as possible.

use std::path::Path;

use crate::cargo_communication::cargo_types::command_creation_details::CommandCreationDetails;
use crate::cargo_communication::cargo_types::command_utils::{
    targets_details_to_args, CommandType,
};
use crate::project_model::target_details::TargetDetails;
use bsp4rs::bsp::{CompileParams, RunParams, TestParams};
use bsp4rs::rust::RustWorkspaceParams;
use std::process::Command;

pub(crate) trait CreateCommand: CommandCreationDetails {
    fn create_requested_command(&self, root: &Path, targets_details: &[TargetDetails]) -> Command {
        let targets_args = targets_details_to_args(targets_details);
        create_requested_command(
            Self::get_command_type(),
            root,
            targets_args,
            self.get_command_arguments(),
        )
    }
}

impl CreateCommand for CompileParams {}

impl CreateCommand for RunParams {}

impl CreateCommand for TestParams {}

impl CreateCommand for RustWorkspaceParams {
    fn create_requested_command(&self, root: &Path, _: &[TargetDetails]) -> Command {
        let mut cmd = create_requested_command(
            Self::get_command_type(),
            root,
            vec![],
            self.get_command_arguments(),
        );
        cmd.env("RUSTC_BOOTSTRAP", "1");
        cmd
    }
}

fn create_requested_command(
    command_type: CommandType,
    root: &Path,
    targets_args: Vec<String>,
    command_args: Vec<String>,
) -> Command {
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root);
    if let CommandType::Test = command_type {
        cmd.arg("+nightly");
    }
    cmd.arg(command_type.to_string());
    cmd.args(targets_args);
    cmd.arg("--message-format=json");
    match command_type {
        CommandType::Build | CommandType::Test | CommandType::Run => {
            cmd.arg("--");
        }
        CommandType::Check => {}
    }
    cmd.args(command_args);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_model::target_details::CargoTargetKind::{Bin, Lib};
    use crate::project_model::DefaultFeature;
    use bsp4rs::bsp::{CompileParams, RunParams, TestParams};
    use bsp4rs::rust::{Feature, RustWorkspaceParams};
    use insta::assert_debug_snapshot;
    use std::collections::BTreeSet;
    use std::ffi::OsStr;

    const TEST_ARGS: [&str; 2] = ["--arg1", "--arg2"];
    const TEST_BIN_NAME: &str = "test_bin1";
    const TEST_LIB_NAME: &str = "test_lib1";
    const TEST_PACKAGE_NAMES: [&str; 2] = ["test_package1", "test_package2"];
    const TEST_ROOT: &str = "/test_root";

    fn default_target_details() -> Vec<TargetDetails> {
        vec![
            TargetDetails {
                name: TEST_BIN_NAME.to_string(),
                kind: Bin,
                package_abs_path: Default::default(),
                package_name: TEST_PACKAGE_NAMES[0].to_string(),
                enabled_features: BTreeSet::from([Feature::default_feature_name()]),
            },
            TargetDetails {
                name: TEST_LIB_NAME.to_string(),
                kind: Lib,
                package_abs_path: Default::default(),
                package_name: TEST_PACKAGE_NAMES[1].to_string(),
                // No `default` feature, means that default features are disabled
                enabled_features: BTreeSet::from([Feature("test_feature1".to_string())]),
            },
        ]
    }

    fn test_compile_params() -> CompileParams {
        CompileParams {
            arguments: Some(vec![TEST_ARGS[0].to_string(), TEST_ARGS[1].to_string()]),
            ..Default::default()
        }
    }

    #[test]
    fn test_compile_params_create_command() {
        let compile_params = test_compile_params();
        let cmd = compile_params
            .create_requested_command(Path::new(TEST_ROOT), &default_target_details());
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r#"
        [
            "build",
            "--package",
            "test_package1",
            "--bin",
            "test_bin1",
            "--package",
            "test_package2",
            "--lib",
            "--features",
            "test_feature1",
            "--no-default-features",
            "--message-format=json",
            "--",
            "--arg1",
            "--arg2",
        ]
        "#);
        assert_eq!(cwd, Path::new(TEST_ROOT));
    }

    fn test_run_params() -> RunParams {
        RunParams {
            arguments: Some(vec![TEST_ARGS[0].to_string(), TEST_ARGS[1].to_string()]),
            ..RunParams::default()
        }
    }

    #[test]
    fn test_run_params_create_command() {
        let run_params = test_run_params();
        let target_details = default_target_details();
        let cmd = run_params.create_requested_command(Path::new(TEST_ROOT), &target_details[0..1]);
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r#"
        [
            "run",
            "--package",
            "test_package1",
            "--bin",
            "test_bin1",
            "--message-format=json",
            "--",
            "--arg1",
            "--arg2",
        ]
        "#);
        assert_eq!(cwd, Path::new(TEST_ROOT));
    }

    fn test_test_params() -> TestParams {
        TestParams {
            arguments: Some(vec![TEST_ARGS[0].to_string(), TEST_ARGS[1].to_string()]),
            ..TestParams::default()
        }
    }

    #[test]
    fn test_test_params_create_command() {
        let test_params = test_test_params();
        let cmd =
            test_params.create_requested_command(Path::new(TEST_ROOT), &default_target_details());
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r#"
        [
            "+nightly",
            "test",
            "--package",
            "test_package1",
            "--bin",
            "test_bin1",
            "--package",
            "test_package2",
            "--lib",
            "--features",
            "test_feature1",
            "--no-default-features",
            "--message-format=json",
            "--",
            "--show-output",
            "-Z",
            "unstable-options",
            "--format=json",
            "--arg1",
            "--arg2",
        ]
        "#);
        assert_eq!(cwd, Path::new(TEST_ROOT));
    }

    #[test]
    fn test_rust_workspace_params_create_command() {
        let rust_workspace_params = RustWorkspaceParams::default();
        let cmd = rust_workspace_params
            .create_requested_command(Path::new(TEST_ROOT), &default_target_details());
        let cwd = cmd.get_current_dir().unwrap();
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let envs: Vec<(&OsStr, Option<&OsStr>)> = cmd.get_envs().collect();

        assert_debug_snapshot!(args, @r#"
        [
            "check",
            "--message-format=json",
            "--workspace",
            "--all-targets",
            "-Z",
            "unstable-options",
            "--keep-going",
        ]
        "#);
        assert_eq!(cwd, Path::new(TEST_ROOT));
        assert_debug_snapshot!(envs, @r#"
        [
            (
                "RUSTC_BOOTSTRAP",
                Some(
                    "1",
                ),
            ),
        ]
        "#);
    }
}

#[cfg(test)]
mod feature_tests {
    use super::*;
    use bsp4rs::rust::Feature;
    use std::collections::BTreeSet;
    use test_case::test_case;

    const TEST_FEATURES: [&str; 4] = ["f1", "f2", "f3", "default"];

    #[test_case(BTreeSet::new(), ""  ;"empty")]
    #[test_case(TEST_FEATURES.iter().map(|&f| Feature::from(f)).collect(),
    "f1, f2, f3" ;
    "non_empty"
    )]
    fn test_get_enabled_features_string(enabled_features: BTreeSet<Feature>, expected: &str) {
        let target_details = TargetDetails {
            enabled_features,
            ..TargetDetails::default()
        };

        let enabled_features_string = target_details
            .get_enabled_features_str()
            .unwrap_or("".to_string());
        assert_eq!(enabled_features_string, expected);
    }
}
