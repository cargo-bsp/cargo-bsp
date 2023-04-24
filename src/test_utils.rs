use std::time::Duration;

use serde_json::to_value;

use crate::bsp_types::notifications::{
    ExitBuild, InitializedBuild, InitializedBuildParams, Notification,
};
use crate::bsp_types::requests::{
    InitializeBuild, InitializeBuildParams, InitializeBuildResult, Request, ShutdownBuild, Sources,
    SourcesParams, SourcesResult,
};
use crate::communication;
use crate::communication::{Connection, Message};
use crate::server::caps::server_capabilities;
use crate::server::config::Config;
use crate::server::Result;

pub struct TestCase {
    pub test_messages: Vec<Message>,
    pub expected_err: String,
    pub expected_send: Vec<Message>,
    pub channel_works_ok: bool,
    pub is_ok: bool,
    pub func_to_test: fn(Connection) -> Result<()>,
}

impl Default for TestCase {
    fn default() -> Self {
        TestCase {
            test_messages: vec![],
            expected_err: "".to_string(),
            expected_send: vec![],
            channel_works_ok: true,
            is_ok: true,
            func_to_test: |_| Ok(()),
        }
    }
}

impl TestCase {
    pub fn test(self) {
        let (client, server) = Connection::memory();

        for msg in self.test_messages {
            client.sender.send(msg).unwrap();
        }

        if !self.channel_works_ok {
            drop(client.sender)
        }

        let resp = (self.func_to_test)(server);
        if self.is_ok {
            assert!(resp.is_ok());
        } else {
            assert!(resp.is_err());
            assert_eq!(self.expected_err, resp.unwrap_err().to_string());
        }

        for msg in self.expected_send {
            assert_eq!(
                msg,
                client
                    .receiver
                    .recv_timeout(Duration::from_secs(1))
                    .unwrap()
            );
        }
        assert!(client
            .receiver
            .recv_timeout(Duration::from_secs(1))
            .is_err());
    }

    pub fn new(channel_works_ok: bool, is_ok: bool) -> Self {
        Self {
            channel_works_ok,
            is_ok,
            ..Self::default()
        }
    }
}

pub fn test_init_params() -> InitializeBuildParams {
    InitializeBuildParams {
        display_name: "TestClient".to_string(),
        ..InitializeBuildParams::default()
    }
}

pub fn test_init_req(params: &InitializeBuildParams, id: i32) -> communication::Request {
    communication::Request {
        id: id.into(),
        method: InitializeBuild::METHOD.to_string(),
        params: to_value(params).unwrap(),
    }
}

pub fn test_init_result(init_params: &InitializeBuildParams) -> InitializeBuildResult {
    let config = Config::from_initialize_params(init_params.clone()).unwrap();
    InitializeBuildResult {
        display_name: "test".to_string(),
        version: "0.0.1".to_string(),
        bsp_version: "2.0.0".to_string(),
        capabilities: server_capabilities(&config),
        data: None,
    }
}

pub fn test_init_resp(params: &InitializeBuildResult, id: i32) -> communication::Response {
    communication::Response {
        id: id.into(),
        result: Some(to_value(params).unwrap()),
        error: None,
    }
}

pub fn test_init_notif() -> communication::Notification {
    communication::Notification {
        method: InitializedBuild::METHOD.to_string(),
        params: to_value(InitializedBuildParams::default()).unwrap(),
    }
}

pub fn test_sources_req(id: i32) -> communication::Request {
    communication::Request {
        id: id.into(),
        method: Sources::METHOD.to_string(),
        params: to_value(SourcesParams::default()).unwrap(),
    }
}

pub fn test_sources_resp(id: i32) -> communication::Response {
    communication::Response {
        id: id.into(),
        result: Some(to_value(SourcesResult::default()).unwrap()),
        error: None,
    }
}

pub fn test_shutdown_req(id: i32) -> communication::Request {
    communication::Request {
        id: id.into(),
        method: ShutdownBuild::METHOD.to_string(),
        params: Default::default(),
    }
}

pub fn test_shutdown_resp(id: i32) -> communication::Response {
    communication::Response {
        id: id.into(),
        result: Some(to_value(()).unwrap()),
        error: None,
    }
}

pub fn test_exit_notif() -> communication::Notification {
    communication::Notification {
        method: ExitBuild::METHOD.to_string(),
        params: Default::default(),
    }
}