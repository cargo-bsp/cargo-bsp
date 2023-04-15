// copy from rust-analyzer

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{unbounded, Receiver, Sender};
use log::info;

use crate::communication;
use crate::communication::{Message, RequestId};
use crate::project_model::ProjectWorkspace;
use crate::server::config::Config;
use crate::server::request_actor::RequestHandle;

pub(crate) type ReqHandler = fn(&mut GlobalState, communication::Response);
pub(crate) type ReqQueue = communication::ReqQueue<(String, Instant), ReqHandler>;

pub(crate) struct GlobalState {
    sender: Sender<Message>,
    req_queue: ReqQueue,
    pub(crate) shutdown_requested: bool,
    pub(crate) config: Config,

    pub(crate) handlers: HashMap<RequestId, RequestHandle>,
    pub(crate) handlers_sender: Sender<Message>,
    pub(crate) handlers_receiver: Receiver<Message>,

    pub(crate) workspace: Arc<ProjectWorkspace>,
}

/// snapshot of server state for request handlers
pub(crate) struct _GlobalStateSnapshot {
    pub(crate) config: Config,
    pub(crate) workspace: Arc<ProjectWorkspace>,
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
            workspace: Arc::new(ProjectWorkspace::default()),
        };
        this.update_workspace_data();
        this
    }

    pub(crate) fn send_notification(&mut self, not: communication::Notification) {
        self.send(not.into());
    }

    pub(crate) fn register_request(
        &mut self,
        request: &communication::Request,
        request_received: Instant,
    ) {
        self.req_queue.incoming.register(
            request.id.clone(),
            (request.method.clone(), request_received),
        );
    }

    pub(crate) fn respond(&mut self, response: communication::Response) {
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

    #[allow(dead_code)]
    pub(crate) fn cancel(&mut self, request_id: RequestId) {
        if let Some(response) = self.req_queue.incoming.cancel(request_id) {
            self.send(response.into());
        }
    }

    fn send(&mut self, message: Message) {
        self.sender.send(message).unwrap()
    }

    // update the workspace data - called when (to be yet added) cargo watch discovers changes
    pub(crate) fn update_workspace_data(&mut self) {
        self.config.update_project_manifest();

        //get a manifest path from config and pass it to new Project Workspace
        self.workspace = Arc::new(ProjectWorkspace::from(
            self.config.workspace_manifest.file.clone(),
        ));
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {}
}
