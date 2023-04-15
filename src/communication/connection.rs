// copy from lsp-server
// #![warn(rust_2018_idioms, unused_lifetimes, semicolon_in_expressions_from_macros)]

use crossbeam_channel::{Receiver, Sender};

use crate::communication::{
    error::ProtocolError,
    msg::{ErrorCode, Message, RequestId, Response},
    stdio,
    stdio::IoThreads,
};

/// Connection is just a pair of channels of LSP messages.
pub struct Connection {
    pub sender: Sender<Message>,
    pub receiver: Receiver<Message>,
}

impl Connection {
    /// Create connection over standard in/standard out.
    ///
    /// Use this to create a real language server.
    pub fn stdio() -> (Connection, IoThreads) {
        let (sender, receiver, io_threads) = stdio::stdio_transport();
        (Connection { sender, receiver }, io_threads)
    }

    /// Starts the initialization process by waiting for an initialize
    /// request from the client. Use this for more advanced customization than
    /// `initialize` can provide.
    ///
    /// Returns the request id and serialized `InitializeParams` from the client.
    pub fn initialize_start(&self) -> Result<(RequestId, serde_json::Value), ProtocolError> {
        loop {
            break match self.receiver.recv() {
                Ok(Message::Request(req)) if req.is_initialize() => Ok((req.id, req.params)),
                // Respond to non-initialize requests with ServerNotInitialized
                Ok(Message::Request(req)) => {
                    let resp = Response::new_err(
                        req.id.clone(),
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {req:?}"),
                    );
                    self.sender.send(resp.into()).unwrap();
                    continue;
                }
                Ok(Message::Notification(n)) if !n.is_exit() => {
                    continue;
                }
                Ok(msg) => Err(ProtocolError(format!(
                    "expected initialize request, got {msg:?}"
                ))),
                Err(e) => Err(ProtocolError(format!(
                    "expected initialize request, got error: {e}"
                ))),
            };
        }
    }

    /// Finishes the initialization process by sending an `InitializeResult` to the client
    pub fn initialize_finish(
        &self,
        initialize_id: RequestId,
        initialize_result: serde_json::Value,
    ) -> Result<(), ProtocolError> {
        let resp = Response::new_ok(initialize_id, initialize_result);
        self.sender.send(resp.into()).unwrap();
        match &self.receiver.recv() {
            Ok(Message::Notification(n)) if n.is_initialized() => Ok(()),
            Ok(msg) => Err(ProtocolError(format!(
                r#"expected initialized notification, got: {msg:?}"#
            ))),
            Err(e) => Err(ProtocolError(format!(
                "expected initialized notification, got error: {e}",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {

    mod test_initialize_start {
        use crossbeam_channel::unbounded;
        use crossbeam_channel::RecvError;
        use serde_json::to_value;

        use crate::bsp_types::notifications::{
            ExitBuild, InitializedBuild, InitializedBuildParams, Notification,
        };
        use crate::bsp_types::requests::{
            CleanCacheResult, InitializeBuild, InitializeBuildParams, Request, Sources,
            SourcesParams,
        };
        use crate::communication;
        use crate::communication::{
            Connection, ErrorCode, Message, ProtocolError, RequestId, Response,
        };

        struct TestCase {
            test_messages: Vec<Message>,
            expected_resp: Result<(RequestId, serde_json::Value), ProtocolError>,
            expected_send: Vec<Message>,
            channel_works_ok: bool,
        }

        fn test_f(test_case: TestCase) {
            let (reader_sender, reader_receiver) = unbounded::<Message>();
            let (writer_sender, writer_receiver) = unbounded::<Message>();
            let conn = Connection {
                sender: writer_sender,
                receiver: reader_receiver,
            };

            for msg in test_case.test_messages {
                assert!(reader_sender.send(msg).is_ok());
            }

            if !test_case.channel_works_ok {
                std::mem::drop(reader_sender)
            }

            let resp = conn.initialize_start();
            assert_eq!(test_case.expected_resp, resp);

            for msg in test_case.expected_send {
                assert_eq!(msg, writer_receiver.recv().unwrap());
            }
        }

        fn test_f_with_initialize(mut test_case: TestCase) {
            let params = InitializeBuildParams {
                display_name: "TestClient1".to_string(),
                ..InitializeBuildParams::default()
            };
            let params_as_value = to_value(params).unwrap();
            let req_id = RequestId::from(234);
            let request = communication::Request {
                id: req_id.clone(),
                method: InitializeBuild::METHOD.to_string(),
                params: params_as_value.clone(),
            };

            test_case.test_messages.push(request.into());
            test_case.expected_resp = Ok((req_id, params_as_value));

            test_f(test_case)
        }

        #[test]
        fn initialize_request() {
            test_f_with_initialize(TestCase {
                test_messages: vec![],
                expected_resp: Err(ProtocolError("something went wrong".into())),
                expected_send: vec![],
                channel_works_ok: true,
            });
        }

        #[test]
        fn not_initialize_request() {
            let params = SourcesParams::default();
            let params_as_value = to_value(params).unwrap();
            let req_id = RequestId::from(123);
            let request = communication::Request {
                id: req_id.clone(),
                method: Sources::METHOD.to_string(),
                params: params_as_value,
            };

            test_f_with_initialize(TestCase {
                test_messages: vec![request.clone().into()],
                expected_resp: Err(ProtocolError("something went wrong".into())),
                expected_send: vec![Response::new_err(
                    req_id,
                    ErrorCode::ServerNotInitialized as i32,
                    format!("expected initialize request, got {:?}", request),
                )
                .into()],
                channel_works_ok: true,
            });
        }

        #[test]
        fn not_exit_notification() {
            let params = InitializedBuildParams::default();
            let params_as_value = to_value(params).unwrap();
            let notification = communication::Notification {
                method: InitializedBuild::METHOD.to_string(),
                params: params_as_value,
            };

            test_f_with_initialize(TestCase {
                test_messages: vec![notification.into()],
                expected_resp: Err(ProtocolError("something went wrong".into())),
                expected_send: vec![],
                channel_works_ok: true,
            });
        }

        #[test]
        fn exit_notification() {
            let notification = communication::Notification {
                method: ExitBuild::METHOD.to_string(),
                params: to_value(()).unwrap(),
            };
            let notification_msg = Message::from(notification);

            test_f(TestCase {
                test_messages: vec![notification_msg.clone()],
                expected_resp: Err(ProtocolError(format!(
                    "expected initialize request, got {:?}",
                    notification_msg
                ))),
                expected_send: vec![],
                channel_works_ok: true,
            });
        }

        #[test]
        fn wrong_msg() {
            let params = CleanCacheResult::default();
            let params_as_value = to_value(params).unwrap();
            let req_id = RequestId::from(123);
            let response = communication::Response {
                id: req_id,
                result: Some(params_as_value),
                error: None,
            };

            test_f(TestCase {
                test_messages: vec![response.clone().into()],
                expected_resp: Err(ProtocolError(format!(
                    "expected initialize request, got {:?}",
                    Message::from(response)
                ))),
                expected_send: vec![],
                channel_works_ok: true,
            });
        }

        #[test]
        fn channel_err() {
            test_f(TestCase {
                test_messages: vec![],
                expected_resp: Err(ProtocolError(format!(
                    "expected initialize request, got error: {}",
                    RecvError {}
                ))),
                expected_send: vec![],
                channel_works_ok: false,
            });
        }
    }

    mod test_initialize_finish {
        use crossbeam_channel::unbounded;
        use crossbeam_channel::RecvError;
        use serde_json::to_value;

        use crate::bsp_types::notifications::{
            InitializedBuild, InitializedBuildParams, Notification,
        };
        use crate::bsp_types::requests::{InitializeBuildResult, Request, Sources, SourcesParams};
        use crate::communication;
        use crate::communication::{Connection, Message, ProtocolError, RequestId, Response};

        struct TestCase {
            test_message: Option<Message>,
            expected_resp: Result<(), ProtocolError>,
        }

        fn test_f(test_case: TestCase) {
            let send_id = RequestId::from(234);
            let send_params = to_value(InitializeBuildResult::default()).unwrap();

            let send_resp = Response {
                id: send_id.clone(),
                result: Some(send_params.clone()),
                error: None,
            };

            let (reader_sender, reader_receiver) = unbounded::<Message>();
            let (writer_sender, writer_receiver) = unbounded::<Message>();
            let conn = Connection {
                sender: writer_sender,
                receiver: reader_receiver,
            };

            match test_case.test_message {
                None => std::mem::drop(reader_sender),
                Some(msg) => {
                    assert!(reader_sender.send(msg).is_ok());
                }
            }

            let resp = conn.initialize_finish(send_id, send_params);
            assert_eq!(Message::from(send_resp), writer_receiver.recv().unwrap());
            assert_eq!(test_case.expected_resp, resp);
        }

        #[test]
        fn initialized_notification() {
            let notification = communication::Notification {
                method: InitializedBuild::METHOD.to_string(),
                params: to_value(InitializedBuildParams::default()).unwrap(),
            };

            test_f(TestCase {
                test_message: Some(notification.into()),
                expected_resp: Ok(()),
            });
        }

        #[test]
        fn wrong_msg() {
            let params = SourcesParams::default();
            let params_as_value = to_value(params).unwrap();
            let req_id = RequestId::from(123);
            let request = communication::Request {
                id: req_id,
                method: Sources::METHOD.to_string(),
                params: params_as_value,
            };

            test_f(TestCase {
                test_message: Some(request.clone().into()),
                expected_resp: Err(ProtocolError(format!(
                    r#"expected initialized notification, got: {:?}"#,
                    Message::from(request)
                ))),
            });
        }

        #[test]
        fn channel_err() {
            test_f(TestCase {
                test_message: None,
                expected_resp: Err(ProtocolError(format!(
                    "expected initialized notification, got error: {}",
                    RecvError {},
                ))),
            });
        }
    }
}
