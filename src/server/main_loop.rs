// copy from rust-analyzer

//! The main loop of `rust-analyzer` responsible for dispatching LSP
//! requests/replies and notifications back to the client.
use std::time::Instant;

use crossbeam_channel::{select, Receiver};

use communication::{Connection, Notification, Request};

use crate::bsp_types::notifications::Notification as _;
use crate::communication::Message;
use crate::logger::log;
use crate::server::config::Config;
use crate::server::dispatch::{NotificationDispatcher, RequestDispatcher};
use crate::server::global_state::GlobalState;
use crate::server::main_loop::Event::{Bsp, FromThread};
use crate::server::{handlers, Result};
use crate::{bsp_types, communication};

pub fn main_loop(config: Config, connection: Connection) -> Result<()> {
    GlobalState::new(connection.sender, config).run(connection.receiver)
}

#[derive(Debug)]
enum Event {
    Bsp(Message),
    FromThread(Message),
}

impl GlobalState {
    fn run(mut self, inbox: Receiver<Message>) -> Result<()> {
        log("server is running");
        if self.config.linked_projects().is_empty() {
            log("bsp cargo failed to discover workspace");
        };

        while let Some(event) = self.next_message(&inbox) {
            if let Bsp(Message::Notification(not)) = &event {
                if not.method == bsp_types::notifications::ExitBuild::METHOD {
                    return Ok(());
                }
            }
            self.handle_message(event)?
        }

        Err("client exited without proper shutdown sequence".into())
    }

    fn next_message(&self, inbox: &Receiver<Message>) -> Option<Event> {
        select! {
            recv(inbox) -> msg =>
                msg.ok().map(Event::Bsp),

            recv(self.handlers_receiver) -> msg =>
                msg.ok().map(Event::FromThread),
        }
    }

    fn handle_message(&mut self, event: Event) -> Result<()> {
        let loop_start = Instant::now();
        log(&format!("{:?} handle_message({:?})", loop_start, event));

        match event {
            Bsp(msg) => match msg {
                Message::Request(req) => self.on_new_request(loop_start, req),
                Message::Notification(not) => self.on_notification(not)?,
                Message::Response(_) => {}
            },
            FromThread(msg) => match &msg {
                Message::Request(_) => {}
                Message::Notification(not) => self.send_notification(not.to_owned()),
                Message::Response(resp) => {
                    self.handlers.remove(&resp.id);
                    self.respond(resp.to_owned())
                }
            },
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
        let mut dispatcher = RequestDispatcher {
            req: Some(req),
            global_state: self,
        };
        dispatcher.on_sync_mut::<bsp_types::requests::ShutdownBuild>(|s, ()| {
            s.shutdown_requested = true;
            Ok(())
        });

        if let RequestDispatcher {
            req: Some(req),
            global_state: this,
        } = &mut dispatcher
        {
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
            .on_sync_mut::<bsp_types::requests::WorkspaceBuildTargets>(
                handlers::handle_workspace_build_targets,
            )
            .on_sync_mut::<bsp_types::requests::Sources>(handlers::handle_sources)
            .on_sync_mut::<bsp_types::requests::Resources>(handlers::handle_resources)
            .on_sync_mut::<bsp_types::requests::JavaExtensions>(handlers::handle_extensions)
            .on_run_cargo::<bsp_types::requests::Compile>()
            .on_run_cargo::<bsp_types::requests::Run>()
            .on_run_cargo::<bsp_types::requests::Test>()
            .on_sync_mut::<bsp_types::requests::Reload>(handlers::handle_reload)
            .finish();
    }

    // Handles an incoming notification.
    fn on_notification(&mut self, not: Notification) -> Result<()> {
        // TODO handle cancel request

        NotificationDispatcher {
            not: Some(not),
            global_state: self,
        }
        .on::<bsp_types::notifications::ExitBuild>(|_, _| {
            log("Got exit notification");
            Ok(())
        })?
        .finish();
        Ok(())
    }
}
