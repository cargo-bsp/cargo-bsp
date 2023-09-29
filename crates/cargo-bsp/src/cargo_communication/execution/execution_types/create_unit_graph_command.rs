//! CreateUnitGraphCommand trait implementation for the Compile/Run/TestParams. The trait allows creating
//! commands regardless if it is the compile, run or test request.
//!
//! The created commands are unit graph commands, which are the same as requested commands (see [`create_command`])
//! but with `--unit-graph -Z unstable-options` flags (only available with `+nightly`).
//! These commands are used to get the number of compilation steps.

use std::path::Path;

use crate::cargo_communication::cargo_types::command_creation_details::CommandCreationDetails;
use crate::cargo_communication::cargo_types::command_utils::{
    targets_details_to_args, CommandType,
};
use crate::project_model::target_details::TargetDetails;
use bsp_types::requests::{CompileParams, RunParams, TestParams};
use std::process::Command;

pub(crate) trait CreateUnitGraphCommand: CommandCreationDetails {
    fn create_unit_graph_command(&self, root: &Path, targets_details: &[TargetDetails]) -> Command {
        let targets_args = targets_details_to_args(targets_details);
        cargo_command_with_unit_graph(Self::get_command_type(), root, targets_args)
    }
}

impl CreateUnitGraphCommand for CompileParams {}

impl CreateUnitGraphCommand for RunParams {}

impl CreateUnitGraphCommand for TestParams {}

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
