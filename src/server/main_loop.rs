//! The main loop responsible for dispatching BSP
//! requests/replies and notifications back to the client.
use std::time::Instant;

use bsp_server::{Connection, ErrorCode, Message, Notification, Request, RequestId, Response};
use crossbeam_channel::{select, Receiver};

use crate::bsp_types;
use crate::bsp_types::notifications::Notification as _;
use crate::server::config::Config;
use crate::server::dispatch::{NotificationDispatcher, RequestDispatcher};
use crate::server::global_state::GlobalState;
use crate::server::main_loop::Event::{Bsp, FromThread};
use crate::server::{handlers, Result};

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
        while let Some(event) = self.next_message(&inbox) {
            if let Bsp(Message::Notification(not)) = &event {
                if not.method == bsp_types::notifications::ExitBuild::METHOD {
                    if !self.shutdown_requested {
                        break;
                    }
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
                this.respond(Response::new_err(
                    req.id.clone(),
                    ErrorCode::InvalidRequest as i32,
                    "Shutdown already requested.".to_owned(),
                ));
                return;
            }
        }

        dispatcher
            .on_sync_mut::<bsp_types::requests::Reload>(handlers::handle_reload)
            .on_sync::<bsp_types::requests::WorkspaceBuildTargets>(
                handlers::handle_workspace_build_targets,
            )
            .on_sync::<bsp_types::requests::Sources>(handlers::handle_sources)
            .on_sync::<bsp_types::requests::Resources>(handlers::handle_resources)
            .on_sync::<bsp_types::requests::JavaExtensions>(handlers::handle_java_extensions)
            .on_sync::<bsp_types::requests::CleanCache>(handlers::handle_clean_cache)
            .on_sync::<bsp_types::requests::DependencyModules>(handlers::handle_dependency_modules)
            .on_sync::<bsp_types::requests::DependencySources>(handlers::handle_dependency_sources)
            .on_sync::<bsp_types::requests::InverseSources>(handlers::handle_inverse_sources)
            .on_sync::<bsp_types::requests::OutputPaths>(handlers::handle_output_paths)
            .on_cargo_run::<bsp_types::requests::Compile>()
            .on_cargo_run::<bsp_types::requests::Run>()
            .on_cargo_run::<bsp_types::requests::Test>()
            .finish();
    }

    /// Handles an incoming notification.
    fn on_notification(&mut self, not: Notification) -> Result<()> {
        NotificationDispatcher {
            not: Some(not),
            global_state: self,
        }
        .on::<lsp_types::notification::Cancel>(|this, params| {
            let id: RequestId = match params.id {
                lsp_types::NumberOrString::Number(id) => id.into(),
                lsp_types::NumberOrString::String(id) => id.into(),
            };
            this.cancel(id);
            Ok(())
        })?
        .finish();
        Ok(())
    }
}
