//! The context or environment in which the server functions.

use std::collections::HashMap;
use std::time::Instant;

use bsp_server;
use bsp_server::{Message, Notification, Request, RequestId, Response};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, info};

use crate::cargo_communication::request_handle::RequestHandle;
use crate::project_model::workspace::ProjectWorkspace;
use crate::server::config::Config;

pub(crate) type ReqHandler = fn(&mut GlobalState, Response);
pub(crate) type ReqQueue = bsp_server::ReqQueue<(String, Instant), ReqHandler>;

/// Stores information about the current state of the project and currently
/// handled/waiting requests.
/// Works as a bridge between handlers and main loop.
pub(crate) struct GlobalState {
    sender: Sender<Message>,
    req_queue: ReqQueue,
    pub(crate) shutdown_requested: bool,
    pub(crate) config: Config,

    pub(crate) handlers: HashMap<RequestId, RequestHandle>,
    pub(crate) handlers_sender: Sender<Message>,
    pub(crate) handlers_receiver: Receiver<Message>,

    pub(crate) workspace: ProjectWorkspace,
}

/// Snapshot of server state for request handlers.
/// At this point, the server is not concurrent (except for the Compile/Run/Test requests),
/// so the data in this snapshot doesn't need to be thread-safe and can be normal references.
/// Prior to the 2023-09-01 commit, these fields were `Arc`s for future planned concurrency support.
/// Since there are no actual plans to develop this server further, we've decided to drop these
/// `Arc`s to simplify the code and fix the problem with ProjectWorkspace not being thread-safe.
pub(crate) struct GlobalStateSnapshot<'a> {
    pub(crate) config: &'a Config,
    pub(crate) workspace: &'a ProjectWorkspace,
}

impl GlobalState {
    pub(crate) fn new(sender: Sender<Message>, config: Config) -> GlobalState {
        let (handlers_sender, handlers_receiver) = unbounded();
        let mut this = GlobalState {
            sender,
            req_queue: ReqQueue::default(),
            shutdown_requested: false,
            config,
            handlers: HashMap::new(),
            handlers_sender,
            handlers_receiver,
            workspace: ProjectWorkspace::default(),
        };
        this.update_workspace_data();
        this
    }

    pub(crate) fn send_notification(&mut self, not: Notification) {
        self.send(not.into());
    }

    pub(crate) fn register_request(&mut self, request: &Request, request_received: Instant) {
        self.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    pub(crate) fn snapshot(&self) -> GlobalStateSnapshot {
        GlobalStateSnapshot {
            config: &self.config,
            workspace: &self.workspace,
        }
    }

    pub(crate) fn respond(&mut self, response: Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(response.id.clone()) {
            let duration = start.elapsed();
            info!(
                "handled {} - ({}) in {:0.2?}",
                method, response.id, duration
            );
            self.send(response.into());
        }
    }

    pub(crate) fn cancel(&mut self, request_id: RequestId) {
        if let Some(response) = self.req_queue.incoming.cancel(request_id) {
            if let Some(handler) = self.handlers.get(&response.id) {
                handler.cancel()
            } else {
                self.send(response.into());
            }
        }
    }

    fn send(&mut self, message: Message) {
        self.sender.send(message).unwrap()
    }

    // update the workspace data - called when (to be yet added) cargo watch discovers changes
    pub(crate) fn update_workspace_data(&mut self) {
        let mutable_config = &mut self.config;
        mutable_config.update_project_manifest();

        match ProjectWorkspace::new(self.config.workspace_manifest.file.clone()) {
            Ok(updated_workspace) => {
                self.workspace = updated_workspace;
            }
            Err(e) => {
                error!("Updating workspace state failed: {}", e);
            }
        }
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {}
}
