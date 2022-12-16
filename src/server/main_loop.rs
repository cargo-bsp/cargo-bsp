// copy from rust-analyzer

//! The main loop of `rust-analyzer` responsible for dispatching LSP
//! requests/replies and notifications back to the client.
use std::time::Instant;

use crossbeam_channel::Receiver;

use communication::{Connection, Notification, Request};

use crate::{bsp_types, communication};
use crate::bsp_types::notifications::Notification as _;
use crate::logger::log;
use crate::server::dispatch::{NotificationDispatcher, RequestDispatcher};
use crate::server::global_state::GlobalState;
use crate::server::{handlers, Result};

pub fn main_loop(connection: Connection) -> Result<()> {
    log("initial config");
    GlobalState::new(connection.sender).run(connection.receiver)
}

impl GlobalState {
    fn run(mut self, inbox: Receiver<communication::Message>) -> Result<()> {
        while let Some(event) = self.next_message(&inbox) {
            if let communication::Message::Notification(not) = &event {
                if not.method == bsp_types::notifications::ExitBuild::METHOD {
                    return Ok(());
                }
            }
            self.handle_message(event)?
        }

        Err("client exited without proper shutdown sequence".into())
    }

    fn next_message(&self, inbox: &Receiver<communication::Message>) -> Option<communication::Message> {
        inbox.recv().ok()
    }

    fn handle_message(&mut self, msg: communication::Message) -> Result<()> {
        let loop_start = Instant::now();
        // NOTE: don't count blocking select! call as a loop-turn time

        log(&format!("{:?} handle_message({:?})", loop_start, msg));

        match msg {
            communication::Message::Request(req) => self.on_new_request(loop_start, req),
            communication::Message::Notification(not) => {
                self.on_notification(not)?;
            }
            communication::Message::Response(_) => {}
        }
        Ok(())
    }
    /// Registers and handles a request. This should only be called once per incoming request.
    fn on_new_request(&mut self, request_received: Instant, req: Request) {
        self.register_request(&req, request_received);
        self.on_request(req);
    }

    /// Handles a request.
    fn on_request(&mut self, req: Request) {
        let mut dispatcher = RequestDispatcher { req: Some(req), global_state: self };
        dispatcher.on_sync_mut::<bsp_types::requests::ShutdownBuild>(|s, ()| {
            s.shutdown_requested = true;
            Ok(())
        });

        if let RequestDispatcher { req: Some(req), global_state: this } = &mut dispatcher {
            if this.shutdown_requested {
                this.respond(communication::Response::new_err(
                    req.id.clone(),
                    communication::ErrorCode::InvalidRequest as i32,
                    "Shutdown already requested.".to_owned(),
                ));
                return;
            }
        }

        dispatcher
            .on_sync_mut::<bsp_types::requests::WorkspaceBuildTargets>(handlers::handle_workspace_build_targets)
            .on_sync_mut::<bsp_types::requests::Sources>(handlers::handle_sources)
            .on_sync_mut::<bsp_types::requests::Resources>(handlers::handle_resources)
            .on_sync_mut::<bsp_types::requests::JavaExtensions>(handlers::handle_extensions)
            .on_sync_mut::<bsp_types::requests::Compile>(handlers::handle_compile)
            .on_sync_mut::<bsp_types::requests::Run>(handlers::handle_run)
            .on_sync_mut::<bsp_types::requests::Test>(handlers::handle_test)
            .on_sync_mut::<bsp_types::requests::Reload>(handlers::handle_reload)
            .finish();
    }

    /// Handles an incoming notification.
    fn on_notification(&mut self, not: Notification) -> Result<()> {
        NotificationDispatcher { not: Some(not), global_state: self }
            .on::<bsp_types::notifications::ExitBuild>(|_, _| {
                log("Got exit notification");
                Ok(())
            })?
            .finish();
        Ok(())
    }
}
