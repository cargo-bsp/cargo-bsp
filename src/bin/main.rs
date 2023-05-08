use std::process::{Child, Command, Stdio};

use cargo_bsp::client::Client;

fn spawn_server() -> Child {
    Command::new("cargo")
        .args(["run", "--bin", "server"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
}

fn main() {
    let mut child = spawn_server();
    Client::new(&mut child);
    println!("Created a client");
}

#[cfg(test)]
mod tests {
    use serde_json::{from_str, to_value};

    use cargo_bsp::bsp_types::notifications::{
        ExitBuild, InitializedBuild, InitializedBuildParams, Notification as _,
    };
    use cargo_bsp::bsp_types::requests::{
        BuildServerCapabilities, Compile, CompileParams, CompileProvider, InitializeBuild,
        InitializeBuildParams, InitializeBuildResult, Request as _, Run, RunParams, RunProvider,
        RunResult, ShutdownBuild, Test, TestParams, TestProvider, TestResult,
        WorkspaceBuildTargetsResult,
    };
    use cargo_bsp::bsp_types::{
        BuildTarget, BuildTargetCapabilities, BuildTargetIdentifier, StatusCode,
    };
    use cargo_bsp::client::Client;
    use cargo_bsp::communication::{Notification, Request, Response};

    use crate::spawn_server;

    fn init_conn(cl: &mut Client) {
        let init_req = create_init_req(2137);
        let proper_resp = create_init_resp(2137);
        let init_notif = create_init_notif();

        cl.send(&serde_json::to_string(&init_req).unwrap());

        let server_resp: Response = from_str(&cl.recv_resp()).unwrap();
        assert_eq!(
            serde_json::to_string(&server_resp).unwrap(),
            serde_json::to_string(&proper_resp).unwrap()
        );

        cl.send(&serde_json::to_string(&init_notif).unwrap());
    }

    fn shutdown_conn(cl: &mut Client) {
        let shutdown_req = create_shutdown_req(2137);
        let proper_resp = create_shutdown_resp(2137);
        let exit_notif = create_exit_notif();

        cl.send(&serde_json::to_string(&shutdown_req).unwrap());

        let server_resp: Response = from_str(&cl.recv_resp()).unwrap();
        assert_eq!(
            serde_json::to_string(&server_resp).unwrap(),
            serde_json::to_string(&proper_resp).unwrap()
        );

        cl.send(&serde_json::to_string(&exit_notif).unwrap());
    }

    #[test]
    fn simple_lifetime() {
        let mut child = spawn_server();
        let mut cl = Client::new(&mut child);
        init_conn(&mut cl);
        shutdown_conn(&mut cl);
        assert_eq!(child.wait().unwrap().code(), Some(0));
    }

    #[test]
    fn immediate_shutdown() {
        let mut child = spawn_server();
        let mut cl = Client::new(&mut child);
        let exit_notif = create_exit_notif();
        cl.send(&serde_json::to_string(&exit_notif).unwrap());
        assert_eq!(child.wait().unwrap().code(), Some(1));
    }

    #[test]
    fn initialize_fail() {
        let mut child = spawn_server();
        let mut cl = Client::new(&mut child);

        let build_workspace_req = create_build_req(2137);
        cl.send(&serde_json::to_string(&build_workspace_req).unwrap());

        let server_resp: Response = from_str(&cl.recv_resp()).unwrap();
        assert_eq!(server_resp.error.unwrap().code, -32002);

        init_conn(&mut cl);
        shutdown_conn(&mut cl);
        assert_eq!(child.wait().unwrap().code(), Some(0));
    }

    #[ignore]
    #[test]
    fn simple_build_req() {
        let mut child = spawn_server();
        let mut cl = Client::new(&mut child);
        init_conn(&mut cl);

        let build_workspace_req = create_build_req(2137);
        let proper_resp = create_build_resp(2137);

        cl.send(&serde_json::to_string(&build_workspace_req).unwrap());

        let server_resp: Response = from_str(&cl.recv_resp()).unwrap();
        assert_eq!(
            serde_json::to_string(&server_resp).unwrap(),
            serde_json::to_string(&proper_resp).unwrap()
        );

        shutdown_conn(&mut cl);
        assert_eq!(child.wait().unwrap().code(), Some(0));
    }

    #[ignore]
    #[test]
    fn simple_run_req() {
        let mut child = spawn_server();
        let mut cl = Client::new(&mut child);
        init_conn(&mut cl);

        let run_req = create_run_req(2137, "2137");
        let proper_resp = create_run_resp(2137, "2137");

        cl.send(&serde_json::to_string(&run_req).unwrap());

        cl.recv_resp(); // LogMessage notification
        cl.recv_resp(); // TaskFinished notification
        let server_resp: Response = from_str(&cl.recv_resp()).unwrap();
        assert_eq!(
            serde_json::to_string(&server_resp).unwrap(),
            serde_json::to_string(&proper_resp).unwrap()
        );

        shutdown_conn(&mut cl);
        assert_eq!(child.wait().unwrap().code(), Some(0));
    }

    #[ignore]
    #[test]
    fn simple_test_req() {
        let mut child = spawn_server();
        let mut cl = Client::new(&mut child);
        init_conn(&mut cl);

        let run_req = create_test_req(2137, "2137");
        let proper_resp = create_test_resp(2137, "2137");

        cl.send(&serde_json::to_string(&run_req).unwrap());

        cl.recv_resp(); // LogMessage notification
        cl.recv_resp(); // TaskFinished notification
        let server_resp: Response = from_str(&cl.recv_resp()).unwrap();
        assert_eq!(
            serde_json::to_string(&server_resp).unwrap(),
            serde_json::to_string(&proper_resp).unwrap()
        );

        shutdown_conn(&mut cl);
        assert_eq!(child.wait().unwrap().code(), Some(0));
    }

    fn create_init_req(id: i32) -> Request {
        let params = InitializeBuildParams {
            display_name: "TestClient".to_string(),
            version: "0.0.1".to_string(),
            bsp_version: "2.0.0".to_string(),
            root_uri: "test".to_string(),
            capabilities: Default::default(),
            data: None,
        };
        Request {
            id: id.into(),
            method: InitializeBuild::METHOD.to_string(),
            params: to_value(params).unwrap(),
        }
    }

    fn create_init_resp(id: i32) -> Response {
        let result = InitializeBuildResult {
            display_name: "test".to_string(),
            version: "0.0.1".to_string(),
            bsp_version: "2.0.0".to_string(),
            capabilities: BuildServerCapabilities {
                compile_provider: Some(CompileProvider {
                    language_ids: vec![],
                }),
                test_provider: Some(TestProvider {
                    language_ids: vec![],
                }),
                run_provider: Some(RunProvider {
                    language_ids: vec![],
                }),
                debug_provider: None,
                inverse_sources_provider: Some(false),
                dependency_sources_provider: Some(false),
                dependency_modules_provider: Some(false),
                resources_provider: Some(false),
                output_paths_provider: Some(false),
                build_target_changed_provider: Some(false),
                jvm_run_environment_provider: Some(false),
                jvm_test_environment_provider: Some(false),
                can_reload: Some(true),
            },
            data: None,
        };
        Response {
            id: id.into(),
            result: Some(to_value(result).unwrap()),
            error: None,
        }
    }

    fn create_init_notif() -> Notification {
        Notification {
            method: InitializedBuild::METHOD.to_string(),
            params: to_value(InitializedBuildParams {}).unwrap(),
        }
    }

    fn create_shutdown_req(id: i32) -> Request {
        Request {
            id: id.into(),
            method: ShutdownBuild::METHOD.to_string(),
            params: Default::default(),
        }
    }

    fn create_shutdown_resp(id: i32) -> Response {
        Response {
            id: id.into(),
            result: None,
            error: None,
        }
    }

    fn create_exit_notif() -> Notification {
        Notification {
            method: ExitBuild::METHOD.to_string(),
            params: Default::default(),
        }
    }

    fn create_build_req(id: i32) -> Request {
        let params = CompileParams {
            targets: vec![BuildTargetIdentifier {
                uri: "main".to_string(),
            }],
            origin_id: "2137".to_owned().into(),
            arguments: vec![],
        };
        Request {
            id: id.into(),
            method: Compile::METHOD.to_string(),
            params: to_value(params).unwrap(),
        }
    }

    fn create_build_resp(id: i32) -> Response {
        let result = WorkspaceBuildTargetsResult {
            targets: vec![BuildTarget {
                id: BuildTargetIdentifier {
                    uri: "//:hello".to_string(),
                },
                display_name: Some("//:hello".to_string()),
                base_directory: None,
                tags: vec![],
                capabilities: BuildTargetCapabilities {
                    can_compile: true,
                    can_test: true,
                    can_run: true,
                    can_debug: false,
                },
                language_ids: vec![],
                dependencies: vec![],
                data: None,
            }],
        };
        Response {
            id: id.into(),
            result: Some(to_value(result).unwrap()),
            error: None,
        }
    }

    fn create_run_req(id: i32, origin_id: &str) -> Request {
        let params = RunParams {
            target: Default::default(),
            origin_id: Some(origin_id.to_string()),
            arguments: vec![],
            data_kind: None,
            data: None,
        };
        Request {
            id: id.into(),
            method: Run::METHOD.to_string(),
            params: to_value(params).unwrap(),
        }
    }

    fn create_run_resp(id: i32, origin_id: &str) -> Response {
        let result = RunResult {
            origin_id: Some(origin_id.to_string()),
            status_code: StatusCode::Ok,
        };
        Response {
            id: id.into(),
            result: Some(to_value(result).unwrap()),
            error: None,
        }
    }

    fn create_test_req(id: i32, origin_id: &str) -> Request {
        let params = TestParams {
            targets: vec![],
            origin_id: Some(origin_id.to_string()),
            arguments: vec![],
            data_kind: None,
            data: None,
        };
        Request {
            id: id.into(),
            method: Test::METHOD.to_string(),
            params: to_value(params).unwrap(),
        }
    }

    fn create_test_resp(id: i32, origin_id: &str) -> Response {
        let result = TestResult {
            origin_id: Some(origin_id.to_string()),
            status_code: StatusCode::Ok,
            data_kind: None,
            data: None,
        };
        Response {
            id: id.into(),
            result: Some(to_value(result).unwrap()),
            error: None,
        }
    }
}
