//! The main loop responsible for dispatching BSP
//! requests/responses and notifications back to the client.

use std::time::Instant;

use bsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use crossbeam_channel::{select, Receiver};

use bsp_types;
use bsp_types::extensions::CancelRequest;
use bsp_types::notifications::Notification as _;

use crate::server::config::Config;
use crate::server::dispatch::{NotificationDispatcher, RequestDispatcher};
use crate::server::global_state::GlobalState;
use crate::server::{handlers, Result};
use crate::utils::request_id::bsp_request_id_to_lsp_request_id;

pub fn main_loop(config: Config, connection: Connection) -> Result<()> {
    GlobalState::new(connection.sender, config).run(connection.receiver)
}

/// Indicates where the message is coming from.
/// Bsp means it comes from the client and the received request/notification should be handled.
/// FromThread means it is from one of the handled requests in the server and the received
/// response/notification should be sent back to the client.
#[derive(Debug)]
enum Event {
    Bsp(Message),
    FromThread(Message),
}

impl GlobalState {
    fn run(mut self, inbox: Receiver<Message>) -> Result<()> {
        while let Some(event) = self.next_message(&inbox) {
            if let Event::Bsp(Message::Notification(not)) = &event {
                if not.method == bsp_types::notifications::OnBuildExit::METHOD {
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
            Event::Bsp(msg) => match msg {
                Message::Request(req) => self.on_new_request(loop_start, req),
                Message::Notification(not) => self.on_notification(not)?,
                Message::Response(_) => {}
            },
            Event::FromThread(msg) => match &msg {
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
        dispatcher.on_sync_mut::<bsp_types::requests::BuildShutdown>(|s, ()| {
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
            .on_sync_mut::<bsp_types::requests::WorkspaceReload>(handlers::handle_reload)
            .on_sync_mut::<bsp_types::extensions::SetCargoFeatures>(
                handlers::handle_set_cargo_features,
            )
            .on_sync::<bsp_types::requests::WorkspaceBuildTargets>(
                handlers::handle_workspace_build_targets,
            )
            .on_sync::<bsp_types::requests::BuildTargetSources>(handlers::handle_sources)
            .on_sync::<bsp_types::requests::BuildTargetResources>(handlers::handle_resources)
            .on_sync::<bsp_types::requests::BuildTargetCleanCache>(handlers::handle_clean_cache)
            .on_sync::<bsp_types::requests::BuildTargetDependencyModules>(
                handlers::handle_dependency_modules,
            )
            .on_sync::<bsp_types::requests::BuildTargetDependencySources>(
                handlers::handle_dependency_sources,
            )
            .on_sync::<bsp_types::requests::BuildTargetInverseSources>(
                handlers::handle_inverse_sources,
            )
            .on_sync::<bsp_types::requests::BuildTargetOutputPaths>(handlers::handle_output_paths)
            .on_sync::<bsp_types::extensions::WorkspaceLibraries>(
                handlers::handle_workspace_libraries,
            )
            .on_sync::<bsp_types::extensions::CargoFeaturesState>(
                handlers::handle_cargo_features_state,
            )
            .on_cargo_run::<bsp_types::requests::BuildTargetCompile>()
            .on_cargo_run::<bsp_types::requests::BuildTargetRun>()
            .on_cargo_run::<bsp_types::requests::BuildTargetTest>()
            .on_cargo_check_run::<bsp_types::extensions::RustWorkspace>()
            .finish();
    }

    /// Handles an incoming notification.
    fn on_notification(&mut self, not: Notification) -> Result<()> {
        NotificationDispatcher {
            not: Some(not),
            global_state: self,
        }
        .on::<CancelRequest>(|this, params| {
            let id = bsp_request_id_to_lsp_request_id(params.id);
            this.cancel(id);
            Ok(())
        })?
        .finish();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod test_shutdown_order {
        use std::path::PathBuf;

        use bsp_server::Connection;

        use bsp_types::requests::BuildClientCapabilities;

        use crate::server::config::Config;
        use crate::server::global_state::GlobalState;
        use crate::server::Result;
        use crate::utils::tests::{
            test_exit_notif, test_shutdown_req, test_shutdown_resp, Channel, ConnectionTestCase,
            FuncReturns,
        };

        enum ShutdownReq {
            Send,
            Omit,
        }

        enum ShutdownNotif {
            Send,
            Omit,
        }

        fn shutdown_order_test(
            mut case: ConnectionTestCase,
            req_action: ShutdownReq,
            notif_action: ShutdownNotif,
        ) {
            let test_id = 234;
            let req = test_shutdown_req(test_id);
            let resp = test_shutdown_resp(test_id);
            let notif = test_exit_notif();

            if let ShutdownReq::Send = req_action {
                case.to_send.push(req.into());
                case.expected_recv.push(resp.into());
            }
            if let ShutdownNotif::Send = notif_action {
                case.to_send.push(notif.into());
            }

            if let FuncReturns::Error = case.func_returns {
                case.expected_err = "client exited without proper shutdown sequence".into();
            }
            case.func_to_test = |server: Connection| -> Result<()> {
                let global_state = GlobalState::new(
                    server.sender,
                    Config::new(PathBuf::from("test"), BuildClientCapabilities::default()),
                );
                global_state.run(server.receiver)
            };

            case.test();
        }

        #[test]
        fn proper_shutdown_order() {
            shutdown_order_test(
                ConnectionTestCase::new(Channel::WorksOk, FuncReturns::Ok),
                ShutdownReq::Send,
                ShutdownNotif::Send,
            );
        }

        #[test]
        fn exit_notif_without_shutdown() {
            shutdown_order_test(
                ConnectionTestCase::new(Channel::WorksOk, FuncReturns::Error),
                ShutdownReq::Omit,
                ShutdownNotif::Send,
            );
        }

        #[test]
        fn channel_err_before_shutdown_req() {
            shutdown_order_test(
                ConnectionTestCase::new(Channel::Disconnects, FuncReturns::Error),
                ShutdownReq::Omit,
                ShutdownNotif::Omit,
            );
        }

        #[test]
        fn channel_err_before_exit_notif() {
            shutdown_order_test(
                ConnectionTestCase::new(Channel::Disconnects, FuncReturns::Error),
                ShutdownReq::Send,
                ShutdownNotif::Omit,
            );
        }
    }
}
