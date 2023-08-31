//! The context or environment in which the server functions.

use std::collections::HashMap;
use std::sync::Arc;
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
    pub(crate) config: Arc<Config>,

    pub(crate) handlers: HashMap<RequestId, RequestHandle>,
    pub(crate) handlers_sender: Sender<Message>,
    pub(crate) handlers_receiver: Receiver<Message>,

    pub(crate) workspace: Arc<ProjectWorkspace>,
}

/// Snapshot of server state for request handlers.
pub(crate) struct GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) workspace: Arc<ProjectWorkspace>,
}

impl GlobalState {
    pub(crate) fn new(sender: Sender<Message>, config: Config) -> GlobalState {
        let (handlers_sender, handlers_receiver) = unbounded();
        let mut this = GlobalState {
            sender,
            req_queue: ReqQueue::default(),
            shutdown_requested: false,
            config: Arc::new(config),
            handlers: HashMap::new(),
            handlers_sender,
            handlers_receiver,
            #[allow(clippy::arc_with_non_send_sync)]
            workspace: Arc::new(ProjectWorkspace::default()),
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
            config: Arc::clone(&self.config),
            workspace: Arc::clone(&self.workspace),
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
        let mutable_config = Arc::make_mut(&mut self.config);
        mutable_config.update_project_manifest();

        #[allow(clippy::arc_with_non_send_sync)]
        match ProjectWorkspace::new(self.config.workspace_manifest.file.clone()) {
            Ok(updated_workspace) => {
                self.workspace = Arc::new(updated_workspace);
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
