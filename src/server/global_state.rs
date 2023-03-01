// copy from rust-analyzer

use std::sync::Arc;
use std::time::Instant;

use crossbeam_channel::{Receiver, Sender, unbounded};

use crate::{bsp_types, communication};
use crate::logger::log;
use crate::project_model::ProjectWorkspace;
use crate::server::config::Config;
use crate::communication::Message;


pub(crate) type ReqHandler = fn(&mut GlobalState, communication::Response);
pub(crate) type ReqQueue = communication::ReqQueue<(String, Instant), ReqHandler>;

pub(crate) struct GlobalState {
    sender: Sender<Message>,
    req_queue: ReqQueue,
    pub(crate) shutdown_requested: bool,
    pub(crate) config: Arc<Config>,
    pub(crate) threads_chan: (Sender<Message>, Receiver<Message>),
    pub(crate) _workspaces: Arc<Vec<ProjectWorkspace>>,
}

/// snapshot of server state for request handlers
pub(crate) struct _GlobalStateSnapshot {
    pub(crate) config: Arc<Config>,
    pub(crate) workspaces: Arc<Vec<ProjectWorkspace>>,
}

impl GlobalState {
    pub(crate) fn new(sender: Sender<communication::Message>, config: Config) -> GlobalState {
        let threads_channel = unbounded();
        let mut this = GlobalState {
            sender,
            req_queue: ReqQueue::default(),
            shutdown_requested: false,
            config: Arc::new(config.clone()),
            threads_chan: threads_channel,
            _workspaces: Arc::new(Vec::new()),
        };
        this.update_configuration(config);
        this
    }

    pub(crate) fn send_notification<N: bsp_types::notifications::Notification>(
        &mut self,
        params: N::Params,
    ) {
        let not = communication::Notification::new(N::METHOD.to_string(), params);
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
            log(&format!(
                "handled {} - ({}) in {:0.2?}",
                method, response.id, duration
            ));
            self.send(response.into());
        }
    }

    #[allow(dead_code)]
    pub(crate) fn cancel(&mut self, request_id: communication::RequestId) {
        if let Some(response) = self.req_queue.incoming.cancel(request_id) {
            self.send(response.into());
        }
    }

    fn send(&mut self, message: communication::Message) {
        self.sender.send(message).unwrap()
    }

    pub(crate) fn update_configuration(&mut self, _config: Config) {
        // TODO
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {}
}
