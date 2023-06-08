use bsp_types::BuildTargetIdentifier;
use log::warn;
use std::io;
use std::path::Path;
use std::process::Command;

use crate::project_model::target_details::TargetDetails;
use bsp_types::requests::{CompileParams, RunParams, TestParams};

const BUILD: &str = "build";
const TEST: &str = "test";
const RUN: &str = "run";

pub trait CreateCommand {
    fn origin_id(&self) -> Option<String>;

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command>;

    fn create_unit_graph_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command>;
}

impl CreateCommand for CompileParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_unit_graph_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details)?;
        let cmd = cargo_command_with_unit_graph(BUILD, root, targets_args);
        Ok(cmd)
    }

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details)?;
        let mut cmd = create_requested_command(BUILD, root, targets_args);
        cmd.args(self.arguments.clone());
        Ok(cmd)
    }
}

impl CreateCommand for RunParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_unit_graph_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(vec![self.target.clone()], get_target_details)?;
        let cmd = cargo_command_with_unit_graph(RUN, root, targets_args);
        Ok(cmd)
    }

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let target_args = target_ids_to_args(vec![self.target.clone()], get_target_details)?;
        let mut cmd = create_requested_command(RUN, root, target_args);
        cmd.args(self.arguments.clone());
        Ok(cmd)
    }
}

impl CreateCommand for TestParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_unit_graph_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details)?;
        let cmd = cargo_command_with_unit_graph(TEST, root, targets_args);
        Ok(cmd)
    }

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details)?;
        let mut cmd = create_requested_command(TEST, root, targets_args);
        cmd.args(["--show-output", "-Z", "unstable-options", "--format=json"])
            .args(self.arguments.clone());
        Ok(cmd)
    }
}

fn target_ids_to_args(
    target_id: Vec<BuildTargetIdentifier>,
    get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
) -> io::Result<Vec<String>> {
    let targets_details: Vec<TargetDetails> = target_id
        .iter()
        .map(|id| {
            get_target_details(id).ok_or_else(|| {
                warn!("Target {:?} not found", id);
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Target {:?} not found", id),
                )
            })
        })
        .collect::<io::Result<Vec<TargetDetails>>>()?;

    let args: Vec<String> = targets_details
        .iter()
        .flat_map(|t| {
            let mut args = Vec::new();
            args.push("--package".to_string());
            args.push(t.package_name.clone());
            args.push(format!("--{}", t.kind));
            args.push(t.name.clone());
            if let Some(features) = t.get_enabled_features_str() {
                args.push(features);
            }
            args
        })
        .collect();

    Ok(args)
}

fn create_requested_command(command_type: &str, root: &Path, targets_args: Vec<String>) -> Command {
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root);
    cmd.arg(command_type);
    cmd.args(targets_args);
    cmd.args(["--message-format=json", "--"]);
    cmd
}

fn cargo_command_with_unit_graph(
    command_type: &str,
    root: &Path,
    targets_args: Vec<String>,
) -> Command {
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root)
        .args([
            "+nightly",
            command_type,
            "--unit-graph",
            "-Z",
            "unstable-options",
        ])
        .args(targets_args);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project_model::cargo_package::Feature;
    use crate::project_model::target_details::CargoTargetKind;
    use bsp_types::requests::{CompileParams, RunParams, TestParams};
    use insta::assert_debug_snapshot;
    use std::collections::BTreeSet;
    use std::ffi::OsStr;

    const TEST_ARGS: [&str; 2] = ["--arg1", "--arg2"];
    const TEST_URI_1: &str = "test_uri1";
    const TEST_URI_2: &str = "test_uri2";
    const TEST_BIN_NAME: &str = "test_bin1";
    const TEST_LIB_NAME: &str = "test_lib1";
    const TEST_PACKAGE_NAMES: [&str; 2] = ["test_package1", "test_package2"];
    const TEST_ROOT: &str = "/test_root";

    fn get_target_details(target_id: &BuildTargetIdentifier) -> Option<TargetDetails> {
        let test_features: BTreeSet<Feature> =
            BTreeSet::from([Feature("test_feature1".to_string())]);
        match target_id.uri.as_str() {
            TEST_URI_1 => Some(TargetDetails {
                name: TEST_BIN_NAME.to_string(),
                kind: CargoTargetKind::Bin,
                package_abs_path: Default::default(),
                package_name: TEST_PACKAGE_NAMES[0].to_string(),
                default_features_disabled: false,
                enabled_features: Default::default(),
            }),
            TEST_URI_2 => Some(TargetDetails {
                name: TEST_LIB_NAME.to_string(),
                kind: CargoTargetKind::Lib,
                package_abs_path: Default::default(),
                package_name: TEST_PACKAGE_NAMES[1].to_string(),
                default_features_disabled: true,
                enabled_features: test_features,
            }),
            _ => None,
        }
    }

    fn default_targets() -> Vec<BuildTargetIdentifier> {
        vec![
            BuildTargetIdentifier {
                uri: TEST_URI_1.to_string(),
            },
            BuildTargetIdentifier {
                uri: TEST_URI_2.to_string(),
            },
        ]
    }

    fn test_compile_params() -> CompileParams {
        CompileParams {
            origin_id: None,
            targets: default_targets(),
            arguments: vec![TEST_ARGS[0].to_string(), TEST_ARGS[1].to_string()],
        }
    }

    #[test]
    fn test_compile_params_create_command() {
        let compile_params = test_compile_params();
        let cmd = compile_params
            .create_requested_command(Path::new(TEST_ROOT), get_target_details)
            .unwrap();
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r###"
        [
            "build",
            "--package",
            "test_package1",
            "--bin",
            "test_bin1",
            "--package",
            "test_package2",
            "--lib",
            "test_lib1",
            "--feature test_feature1",
            "--message-format=json",
            "--",
            "--arg1",
            "--arg2",
        ]
        "###);
        assert_eq!(cwd, Path::new(TEST_ROOT));
    }

    fn test_run_params() -> RunParams {
        RunParams {
            target: BuildTargetIdentifier {
                uri: TEST_URI_1.to_string(),
            },
            arguments: vec![TEST_ARGS[0].to_string(), TEST_ARGS[1].to_string()],
            ..RunParams::default()
        }
    }

    #[test]
    fn test_run_params_create_command() {
        let run_params = test_run_params();
        let cmd = run_params
            .create_requested_command(Path::new(TEST_ROOT), get_target_details)
            .unwrap();
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r###"
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
        "###);
        assert_eq!(cwd, Path::new(TEST_ROOT));
    }

    fn test_test_params() -> TestParams {
        TestParams {
            targets: default_targets(),
            arguments: vec![TEST_ARGS[0].to_string(), TEST_ARGS[1].to_string()],
            ..TestParams::default()
        }
    }

    #[test]
    fn test_test_params_create_command() {
        let test_params = test_test_params();
        let cmd = test_params
            .create_requested_command(Path::new(TEST_ROOT), get_target_details)
            .unwrap();
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r###"
        [
            "test",
            "--package",
            "test_package1",
            "--bin",
            "test_bin1",
            "--package",
            "test_package2",
            "--lib",
            "test_lib1",
            "--feature test_feature1",
            "--message-format=json",
            "--",
            "--show-output",
            "-Z",
            "unstable-options",
            "--format=json",
            "--arg1",
            "--arg2",
        ]
        "###);
        assert_eq!(cwd, Path::new(TEST_ROOT));
    }
}
