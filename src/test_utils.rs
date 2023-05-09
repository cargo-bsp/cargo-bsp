use std::time::Duration;

use bsp_server;
use bsp_server::{Connection, Message};
use crossbeam_channel::Receiver;
use serde_json::to_value;

use crate::bsp_types::notifications::{
    ExitBuild, InitializedBuild, InitializedBuildParams, Notification,
};
use crate::bsp_types::requests::{
    InitializeBuild, InitializeBuildParams, InitializeBuildResult, Request, ShutdownBuild, Sources,
    SourcesParams, SourcesResult,
};
use crate::server::Result;

pub enum Channel {
    WorksOk,
    Disconnects,
}

pub enum FuncReturns {
    Ok,
    Error,
}

pub struct ConnectionTestCase {
    pub to_send: Vec<Message>,
    pub expected_err: String,
    pub expected_recv: Vec<Message>,
    pub channel_state: Channel,
    pub func_returns: FuncReturns,
    pub func_to_test: fn(Connection) -> Result<()>,
}

impl ConnectionTestCase {
    pub fn new(channel_state: Channel, func_returns: FuncReturns) -> Self {
        Self {
            to_send: vec![],
            expected_err: String::default(),
            expected_recv: vec![],
            channel_state,
            func_returns,
            func_to_test: |_| Ok(()),
        }
    }

    pub fn test(self) {
        let (client, server) = Connection::memory();

        for msg in self.to_send {
            client.sender.send(msg).unwrap();
        }

        if let Channel::Disconnects = self.channel_state {
            drop(client.sender)
        }

        let resp = (self.func_to_test)(server);
        if let FuncReturns::Ok = self.func_returns {
            assert!(resp.is_ok());
        } else {
            assert!(resp.is_err());
            assert_eq!(self.expected_err, resp.unwrap_err().to_string());
        }

        for msg in self.expected_recv {
            assert_eq!(
                msg,
                client
                    .receiver
                    .recv_timeout(Duration::from_millis(100))
                    .unwrap()
            );
        }
        assert!(client.receiver.is_empty());
    }
}

pub fn no_more_msg<T>(receiver: Receiver<T>) {
    assert!(receiver.recv_timeout(Duration::from_millis(200)).is_err());
}

pub fn test_init_params() -> InitializeBuildParams {
    InitializeBuildParams {
        display_name: "TestClient".to_string(),
        ..InitializeBuildParams::default()
    }
}

pub fn test_init_req(params: &InitializeBuildParams, id: i32) -> bsp_server::Request {
    bsp_server::Request {
        id: id.into(),
        method: InitializeBuild::METHOD.to_string(),
        params: to_value(params).unwrap(),
    }
}

pub fn test_init_resp(params: &InitializeBuildResult, id: i32) -> bsp_server::Response {
    bsp_server::Response {
        id: id.into(),
        result: Some(to_value(params).unwrap()),
        error: None,
    }
}

pub fn test_init_notif() -> bsp_server::Notification {
    bsp_server::Notification {
        method: InitializedBuild::METHOD.to_string(),
        params: to_value(InitializedBuildParams::default()).unwrap(),
    }
}

pub fn test_sources_req(id: i32) -> bsp_server::Request {
    bsp_server::Request {
        id: id.into(),
        method: Sources::METHOD.to_string(),
        params: to_value(SourcesParams::default()).unwrap(),
    }
}

pub fn test_sources_resp(id: i32) -> bsp_server::Response {
    bsp_server::Response {
        id: id.into(),
        result: Some(to_value(SourcesResult::default()).unwrap()),
        error: None,
    }
}

pub fn test_shutdown_req(id: i32) -> bsp_server::Request {
    bsp_server::Request {
        id: id.into(),
        method: ShutdownBuild::METHOD.to_string(),
        params: Default::default(),
    }
}

pub fn test_shutdown_resp(id: i32) -> bsp_server::Response {
    bsp_server::Response {
        id: id.into(),
        result: Some(to_value(()).unwrap()),
        error: None,
    }
}

pub fn test_exit_notif() -> bsp_server::Notification {
    bsp_server::Notification {
        method: ExitBuild::METHOD.to_string(),
        params: Default::default(),
    }
}
