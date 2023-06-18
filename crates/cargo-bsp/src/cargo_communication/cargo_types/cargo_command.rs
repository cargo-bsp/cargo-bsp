use bsp_types::BuildTargetIdentifier;
use log::warn;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use std::io;
use std::path::Path;

use crate::project_model::target_details::CargoTargetKind::Lib;
use crate::project_model::target_details::TargetDetails;
use bsp_types::requests::{CompileParams, RunParams, TestParams};
use std::process::Command;

#[derive(Debug, Deserialize_enum_str, Serialize_enum_str, Clone)]
#[serde(rename_all = "camelCase")]
enum CommandType {
    Build,
    Test,
    Run,
}

const FEATURE_FLAG: &str = "--feature";

pub trait CreateCommand {
    fn origin_id(&self) -> Option<String>;

    fn create_unit_graph_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command>;

    fn create_requested_command(
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
        let cmd = cargo_command_with_unit_graph(CommandType::Build, root, targets_args);
        Ok(cmd)
    }

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details)?;
        let mut cmd = create_requested_command(CommandType::Build, root, targets_args);
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
        let cmd = cargo_command_with_unit_graph(CommandType::Run, root, targets_args);
        Ok(cmd)
    }

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let target_args = target_ids_to_args(vec![self.target.clone()], get_target_details)?;
        let mut cmd = create_requested_command(CommandType::Run, root, target_args);
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
        let cmd = cargo_command_with_unit_graph(CommandType::Test, root, targets_args);
        Ok(cmd)
    }

    fn create_requested_command(
        &self,
        root: &Path,
        get_target_details: impl Fn(&BuildTargetIdentifier) -> Option<TargetDetails>,
    ) -> io::Result<Command> {
        let targets_args = target_ids_to_args(self.targets.clone(), get_target_details)?;
        let mut cmd = create_requested_command(CommandType::Test, root, targets_args);
        cmd.args(["--show-output", "-Z", "unstable-options", "--format=json"])
            .args(self.arguments.clone());
        Ok(cmd)
    }
}

impl TargetDetails {
    pub fn get_enabled_features_str(&self) -> Option<String> {
        match self.enabled_features.is_empty() {
            true => None,
            false => Some(
                self.enabled_features
                    .iter()
                    .map(|f| f.0.clone())
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        }
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
            let mut loc_args = Vec::new();
            loc_args.push("--package".to_string());
            loc_args.push(t.package_name.clone());
            if t.kind == Lib {
                loc_args.push("--lib".to_string());
            } else {
                loc_args.push(format!("--{}", t.kind));
                loc_args.push(t.name.clone());
            }
            if let Some(features) = t.get_enabled_features_str() {
                loc_args.push(FEATURE_FLAG.to_string());
                loc_args.push(features);
            }
            loc_args
        })
        .collect();

    Ok(args)
}

fn create_requested_command(
    command_type: CommandType,
    root: &Path,
    targets_args: Vec<String>,
) -> Command {
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root);
    if let CommandType::Test = command_type {
        cmd.arg("+nightly");
    }
    cmd.arg(command_type.to_string());
    cmd.args(targets_args);
    cmd.args(["--message-format=json", "--"]);
    cmd
}

fn cargo_command_with_unit_graph(
    command_type: CommandType,
    root: &Path,
    targets_args: Vec<String>,
) -> Command {
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root)
        .args([
            "+nightly",
            command_type.to_string().as_str(),
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
                package_name: TEST_PACKAGE_NAMES[0].to_string(),
                ..Default::default()
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
            "--feature",
            "test_feature1",
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
            "+nightly",
            "test",
            "--package",
            "test_package1",
            "--bin",
            "test_bin1",
            "--package",
            "test_package2",
            "--lib",
            "--feature",
            "test_feature1",
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

#[cfg(test)]
mod feature_tests {
    use super::*;
    use crate::project_model::cargo_package::Feature;
    use std::collections::BTreeSet;
    use test_case::test_case;

    const TEST_FEATURES: [&str; 3] = ["test_feature1", "test_feature2", "test_feature3"];

    #[test_case(BTreeSet::new(), ""  ;"empty")]
    #[test_case(TEST_FEATURES.iter().map(|f| Feature(f.to_string())).collect(),
    "test_feature1, test_feature2, test_feature3" ;
    "non_empty"
    )]
    fn test_get_enabled_features_string(enabled_features: BTreeSet<Feature>, expected: &str) {
        let target_details = TargetDetails {
            default_features_disabled: false,
            enabled_features,
            ..TargetDetails::default()
        };

        let enabled_features_string = target_details
            .get_enabled_features_str()
            .unwrap_or("".to_string());
        assert_eq!(enabled_features_string, expected);
    }
}
