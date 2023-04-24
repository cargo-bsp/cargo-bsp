//! The main loop responsible for dispatching BSP
//! requests/replies and notifications back to the client.
use std::time::Instant;

use crossbeam_channel::{select, Receiver};

use communication::{Connection, Notification, Request};

use crate::bsp_types::notifications::Notification as _;
use crate::communication::Message;
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
                this.respond(communication::Response::new_err(
                    req.id.clone(),
                    communication::ErrorCode::InvalidRequest as i32,
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
            let id: communication::RequestId = match params.id {
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

#[cfg(test)]
mod tests {
    mod test_shutdown_order {
        use std::path::PathBuf;

        use crate::bsp_types::requests::BuildClientCapabilities;
        use crate::communication::Connection;
        use crate::server::config::Config;
        use crate::server::global_state::GlobalState;
        use crate::server::Result;
        use crate::test_utils::{test_exit_notif, test_shutdown_req, test_shutdown_resp, TestCase};

        struct ShutdownTestCase {
            case: TestCase,
            add_req: bool,
            add_notif: bool,
        }

        fn shutdown_order_test(mut test_case: ShutdownTestCase) {
            let test_id = 234;
            let req = test_shutdown_req(test_id);
            let resp = test_shutdown_resp(test_id);
            let notif = test_exit_notif();

            if test_case.add_req {
                test_case.case.test_messages.push(req.into());
                test_case.case.expected_send.push(resp.into());
            }
            if test_case.add_notif {
                test_case.case.test_messages.push(notif.into());
            }

            test_case.case.expected_err = "client exited without proper shutdown sequence".into();
            test_case.case.func_to_test = |server: Connection| -> Result<()> {
                let global_state = GlobalState::new(
                    server.sender,
                    Config::new(PathBuf::from("test"), BuildClientCapabilities::default()),
                );
                global_state.run(server.receiver)
            };

            test_case.case.test();
        }

        #[test]
        fn proper_shutdown_order() {
            shutdown_order_test(ShutdownTestCase {
                case: TestCase::new(true, true),
                add_req: true,
                add_notif: true,
            });
        }

        #[test]
        fn exit_notif_without_shutdown() {
            shutdown_order_test(ShutdownTestCase {
                case: TestCase::new(true, false),
                add_req: false,
                add_notif: true,
            });
        }

        #[test]
        fn channel_err_before_shutdown_req() {
            shutdown_order_test(ShutdownTestCase {
                case: TestCase::new(false, false),
                add_req: false,
                add_notif: false,
            });
        }

        #[test]
        fn channel_err_before_exit_notif() {
            shutdown_order_test(ShutdownTestCase {
                case: TestCase::new(false, false),
                add_req: true,
                add_notif: false,
            });
        }
    }
}
