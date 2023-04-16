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
        use std::time::Duration;

        use crossbeam_channel::unbounded;
        use serde_json::to_value;

        use crate::bsp_types::notifications::{ExitBuild, Notification};
        use crate::bsp_types::requests::{BuildClientCapabilities, Request, ShutdownBuild};
        use crate::communication;
        use crate::communication::{Message, RequestId, Response};
        use crate::server::config::Config;
        use crate::server::global_state::GlobalState;

        struct TestCase {
            test_messages: Vec<Message>,
            expected_send: Vec<Message>,
            is_ok: bool,
        }

        fn test_f(test_case: TestCase) {
            let (reader_sender, reader_receiver) = unbounded::<Message>();
            let (writer_sender, writer_receiver) = unbounded::<Message>();

            let global_state = GlobalState::new(
                writer_sender,
                Config::new(PathBuf::from("test"), BuildClientCapabilities::default()),
            );

            for msg in test_case.test_messages {
                assert!(reader_sender.send(msg).is_ok());
            }
            let notification = communication::Notification {
                method: ExitBuild::METHOD.into(),
                params: to_value(()).unwrap(),
            };
            assert!(reader_sender.send(notification.into()).is_ok());

            let result = global_state.run(reader_receiver);
            if test_case.is_ok {
                assert!(result.is_ok());
            } else {
                assert!(result.is_err());
                assert_eq!(
                    "client exited without proper shutdown sequence".to_string(),
                    result.unwrap_err().to_string()
                );
            }

            for msg in test_case.expected_send {
                assert_eq!(
                    msg,
                    writer_receiver
                        .recv_timeout(Duration::from_secs(1))
                        .unwrap()
                );
            }
            assert!(writer_receiver
                .recv_timeout(Duration::from_secs(1))
                .is_err());
        }

        #[test]
        fn exit_notif_without_shutdown() {
            test_f(TestCase {
                test_messages: vec![],
                expected_send: vec![],
                is_ok: false,
            });
        }

        #[test]
        fn proper_shutdown_order() {
            let req_id = RequestId::from(123);
            let request = communication::Request {
                id: req_id.clone(),
                method: ShutdownBuild::METHOD.to_string(),
                params: to_value(()).unwrap(),
            };

            test_f(TestCase {
                test_messages: vec![request.into()],
                expected_send: vec![Response::new_ok(req_id, to_value(()).unwrap()).into()],
                is_ok: true,
            });
        }
    }
}
