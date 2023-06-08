use std::path::PathBuf;
use std::process::Command;

use bsp_types::requests::{CompileParams, RunParams, TestParams};

fn cargo_command_with_unit_graph(command_type: &str, root: PathBuf) -> Command {
    // TODO add appropriate build target to arguments
    let mut cmd = Command::new(toolchain::cargo());
    cmd.current_dir(root).args([
        "+nightly",
        command_type,
        "--unit-graph",
        "-Z",
        "unstable-options",
    ]);
    cmd
}

pub trait CreateCommand {
    fn origin_id(&self) -> Option<String>;

    fn create_unit_graph_command(&self, root: PathBuf) -> Command;

    fn create_requested_command(&self, root: PathBuf) -> Command;
}

impl CreateCommand for CompileParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_unit_graph_command(&self, root: PathBuf) -> Command {
        cargo_command_with_unit_graph("build", root)
    }

    fn create_requested_command(&self, root: PathBuf) -> Command {
        // TODO add appropriate build target to arguments
        let mut cmd = Command::new(toolchain::cargo());
        cmd.current_dir(root)
            .args([
                "build",
                "--message-format=json",
                self.targets[0].clone().uri.as_str(),
            ])
            .args(self.arguments.clone());
        cmd
    }
}

impl CreateCommand for RunParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_unit_graph_command(&self, root: PathBuf) -> Command {
        cargo_command_with_unit_graph("run", root)
    }

    fn create_requested_command(&self, root: PathBuf) -> Command {
        // TODO add appropriate build target to arguments
        let mut cmd = Command::new(toolchain::cargo());
        cmd.current_dir(root)
            .args([
                "run",
                "--message-format=json",
                self.target.clone().uri.as_str(),
            ])
            .args(self.arguments.clone());
        cmd
    }
}

impl CreateCommand for TestParams {
    fn origin_id(&self) -> Option<String> {
        self.origin_id.clone()
    }

    fn create_unit_graph_command(&self, root: PathBuf) -> Command {
        cargo_command_with_unit_graph("test", root)
    }

    fn create_requested_command(&self, root: PathBuf) -> Command {
        // TODO add appropriate build target to arguments
        let mut cmd = Command::new(toolchain::cargo());
        cmd.current_dir(root)
            .args([
                "test",
                "--message-format=json",
                "--",
                "--show-output",
                "-Z",
                "unstable-options",
                "--format=json",
            ])
            .args(self.arguments.clone());
        cmd
    }
}
