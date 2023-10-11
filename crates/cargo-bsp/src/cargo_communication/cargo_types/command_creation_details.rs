use crate::cargo_communication::cargo_types::command_utils::CommandType;
use bsp_types::bsp::{CompileParams, RunParams, TestParams};
use bsp_types::rust::RustWorkspaceParams;

pub(crate) trait CommandCreationDetails {
    fn get_command_arguments(&self) -> Vec<String>;

    fn get_command_type() -> CommandType;
}

impl CommandCreationDetails for CompileParams {
    fn get_command_arguments(&self) -> Vec<String> {
        self.arguments.clone().unwrap_or_default()
    }

    fn get_command_type() -> CommandType {
        CommandType::Build
    }
}

impl CommandCreationDetails for RunParams {
    fn get_command_arguments(&self) -> Vec<String> {
        self.arguments.clone().unwrap_or_default()
    }

    fn get_command_type() -> CommandType {
        CommandType::Run
    }
}

impl CommandCreationDetails for TestParams {
    fn get_command_arguments(&self) -> Vec<String> {
        let mut args = vec![
            "--show-output".into(),
            "-Z".into(),
            "unstable-options".into(),
            "--format=json".into(),
        ];
        args.extend(self.arguments.clone().unwrap_or_default());
        args
    }

    fn get_command_type() -> CommandType {
        CommandType::Test
    }
}

impl CommandCreationDetails for RustWorkspaceParams {
    fn get_command_arguments(&self) -> Vec<String> {
        vec![
            "--workspace".into(),
            "--all-targets".into(),
            "-Z".into(),
            "unstable-options".into(),
            "--keep-going".into(),
        ]
    }

    fn get_command_type() -> CommandType {
        CommandType::Check
    }
}
