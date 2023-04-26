use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use bsp_server;
use bsp_server::{Message, Notification, Request, RequestId, Response};
use crossbeam_channel::{unbounded, Receiver, Sender};
use log::{error, info};

use crate::project_model::workspace::ProjectWorkspace;
use crate::server::config::Config;
use crate::server::request_actor::RequestHandle;

pub(crate) type ReqHandler = fn(&mut GlobalState, Response);
pub(crate) type ReqQueue = bsp_server::ReqQueue<(String, Instant), ReqHandler>;

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

/// snapshot of server state for request handlers
pub(crate) struct GlobalStateSnapshot {
    pub(crate) _config: Arc<Config>,
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
            _config: Arc::clone(&self.config),
            workspace: Arc::clone(&self.workspace),
        }
    }

    pub(crate) fn respond(&mut self, response: Response) {
        if let Some((method, start)) = self.req_queue.incoming.complete(response.id.clone()) {
            if let Some(err) = &response.error {
                if err.message.starts_with("server panicked") {
                    // self.poke_rust_analyzer_developer(format!("{}, check the log", err.message))
                }
            }

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
            }
            self.send(response.into());
        }
    }

    fn send(&mut self, message: Message) {
        self.sender.send(message).unwrap()
    }

    // update the workspace data - called when (to be yet added) cargo watch discovers changes
    pub(crate) fn update_workspace_data(&mut self) {
        let mutable_config = Arc::make_mut(&mut self.config);
        mutable_config.update_project_manifest();

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
