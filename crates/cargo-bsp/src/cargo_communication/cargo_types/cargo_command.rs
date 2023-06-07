use bsp_types::BuildTargetIdentifier;
use log::warn;
use std::path::Path;
use std::process::Command;

use crate::project_model::target_details::TargetDetails;
use bsp_types::requests::{CompileParams, RunParams, TestParams};

pub trait CreateCommand {
    fn origin_id(&self) -> Option<String>;

    fn create_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> Command;
}

fn target_ids_to_args(
    target_id: Vec<BuildTargetIdentifier>,
    get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
) -> String {
    let targets_details = target_id.iter().map(|id| {
        get_target_details(id).unwrap_or_else(|| {
            warn!("Target details not found for: {:?}", id);
            TargetDetails::default()
        })
    });
    targets_details
        .map(|t| {
            format!(
                "--package {} --{} {} {}",
                t.package_name.as_str(),
                t.kind.to_string().as_str(),
                t.name.as_str(),
                t.get_enabled_features_str().as_str(),
            )
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn create_exec_command(exec_cmd: &str, root: &Path, targets_args: &str) -> Command {
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root);
    cmd.args([exec_cmd, "--message-format=json", targets_args, "--"]);
    cmd
}

impl CreateCommand for CompileParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> Command {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details);
        let mut cmd = create_exec_command("build", root, targets_args.as_str());
        cmd.args(self.arguments.clone());
        cmd
    }
}

impl CreateCommand for RunParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> Command {
        let target_args = target_ids_to_args(vec![self.target.clone()], get_target_details);
        let mut cmd = create_exec_command("run", root, target_args.as_str());
        cmd.args(self.arguments.clone());
        cmd
    }
}

impl CreateCommand for TestParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> Command {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details);
        let mut cmd = create_exec_command("test", root, targets_args.as_str());
        cmd.args(["--show-output", "-Z", "unstable-options", "--format=json"])
            .args(self.arguments.clone());
        cmd
    }
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
            arguments: vec![TEST_ARGS[0].to_string(), TEST_ARGS[0].to_string()],
        }
    }

    #[test]
    fn test_compile_params_create_command() {
        let compile_params = test_compile_params();
        let cmd = compile_params.create_command(Path::new(TEST_ROOT), get_target_details);
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r###"
        [
            "build",
            "--message-format=json",
            "--package test_package1 --bin test_bin1  --package test_package2 --lib test_lib1 --feature test_feature1",
            "--",
            "--arg1",
            "--arg1",
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
        let cmd = run_params.create_command(Path::new(TEST_ROOT), get_target_details);
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r###"
        [
            "run",
            "--message-format=json",
            "--package test_package1 --bin test_bin1 ",
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
        let cmd = test_params.create_command(Path::new(TEST_ROOT), get_target_details);
        let args: Vec<&OsStr> = cmd.get_args().collect();
        let cwd = cmd.get_current_dir().unwrap();

        assert_debug_snapshot!(args, @r###"
        [
            "test",
            "--message-format=json",
            "--package test_package1 --bin test_bin1  --package test_package2 --lib test_lib1 --feature test_feature1",
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
