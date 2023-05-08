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

    /// Creates a pair of connected connections.
    ///
    /// Use this for testing.
    pub fn memory() -> (Connection, Connection) {
        let (s1, r1) = crossbeam_channel::unbounded();
        let (s2, r2) = crossbeam_channel::unbounded();
        (
            Connection {
                sender: s1,
                receiver: r2,
            },
            Connection {
                sender: s2,
                receiver: r1,
            },
        )
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
