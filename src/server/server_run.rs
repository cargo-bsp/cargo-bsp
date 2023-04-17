use log::info;

use crate::bsp_types::requests::{InitializeBuildParams, InitializeBuildResult};
use crate::communication::Connection;
use crate::server;
use crate::server::caps::server_capabilities;
use crate::server::config::Config;
use crate::server::{from_json, Result};

pub fn run_server() -> Result<()> {
    info!("server will start");

    let (connection, io_threads) = Connection::stdio();

    let config = initialize(&connection)?;

    server::main_loop(config, connection)?;

    io_threads.join()?;
    info!("server did shut down");
    Ok(())
}

fn initialize(connection: &Connection) -> Result<Config> {
    let (initialize_id, initialize_params) = connection.initialize_start()?;
    let initialize_params =
        from_json::<InitializeBuildParams>("InitializeParams", &initialize_params)?;

    let config = Config::from_initialize_params(initialize_params)?;
    let initialize_result = create_initialize_result(&config);

    connection.initialize_finish(
        initialize_id,
        serde_json::to_value(initialize_result).unwrap(),
    )?;

    Ok(config)
}

fn create_initialize_result(config: &Config) -> InitializeBuildResult {
    InitializeBuildResult {
        display_name: "test".to_string(),
        version: "0.0.1".to_string(),
        bsp_version: "2.0.0".to_string(),
        capabilities: server_capabilities(config),
        data: None,
    }
}

#[cfg(test)]
mod tests {
    mod test_initialize {
        use std::time::Duration;

        use crossbeam_channel::unbounded;
        use crossbeam_channel::RecvError;
        use serde_json::to_value;

        use crate::bsp_types::notifications::{
            ExitBuild, InitializedBuild, InitializedBuildParams, Notification,
        };
        use crate::bsp_types::requests::{
            CleanCacheResult, InitializeBuild, InitializeBuildParams, Request, Sources,
            SourcesParams,
        };
        use crate::communication;
        use crate::communication::{Connection, ErrorCode, Message, RequestId, Response};
        use crate::server::config::Config;
        use crate::server::server_run::{create_initialize_result, initialize};

        struct TestCase {
            test_messages: Vec<Message>,
            expected_err: String,
            expected_send: Vec<Message>,
            channel_works_ok: bool,
            is_ok: bool,
        }

        fn test_f(test_case: TestCase) {
            let (reader_sender, reader_receiver) = unbounded::<Message>();
            let (writer_sender, writer_receiver) = unbounded::<Message>();
            let conn = Connection {
                sender: writer_sender,
                receiver: reader_receiver,
            };

            for msg in test_case.test_messages {
                assert!(reader_sender.send(msg).is_ok());
            }

            if !test_case.channel_works_ok {
                drop(reader_sender)
            }

            let resp = initialize(&conn);
            if test_case.is_ok {
                assert!(resp.is_ok());
            } else {
                assert!(resp.is_err());
                assert_eq!(test_case.expected_err, resp.unwrap_err().to_string());
            }

            for msg in test_case.expected_send {
                assert_eq!(msg, writer_receiver.recv().unwrap());
            }
            assert!(writer_receiver
                .recv_timeout(Duration::from_secs(1))
                .is_err());
        }

        fn test_f_with_initialize(mut test_case: TestCase, with_notif: bool) {
            let test_id = 234;

            let init_params = InitializeBuildParams {
                display_name: "TestClient1".to_string(),
                ..InitializeBuildParams::default()
            };
            let config = Config::from_initialize_params(init_params.clone()).unwrap();

            let init_req = communication::Request {
                id: RequestId::from(test_id),
                method: InitializeBuild::METHOD.to_string(),
                params: to_value(init_params).unwrap(),
            };
            let init_resp = Response {
                id: RequestId::from(test_id),
                result: Some(to_value(create_initialize_result(&config)).unwrap()),
                error: None,
            };
            let init_notif = communication::Notification {
                method: InitializedBuild::METHOD.to_string(),
                params: to_value(InitializedBuildParams::default()).unwrap(),
            };

            if with_notif {
                test_case.test_messages.push(init_req.into());
                test_case.test_messages.push(init_notif.into());
            } else {
                let mut temp = vec![init_req.into()];
                temp.append(&mut test_case.test_messages);
                test_case.test_messages = temp;
            }
            test_case.expected_send.push(init_resp.into());

            test_f(test_case)
        }

        #[test]
        fn proper_initialize() {
            test_f_with_initialize(
                TestCase {
                    test_messages: vec![],
                    expected_err: "".to_string(),
                    expected_send: vec![],
                    channel_works_ok: true,
                    is_ok: true,
                },
                true,
            );
        }

        #[test]
        fn wrong_req_before_init_req() {
            let req_id = RequestId::from(123);
            let request = communication::Request {
                id: req_id.clone(),
                method: Sources::METHOD.to_string(),
                params: to_value(SourcesParams::default()).unwrap(),
            };

            test_f_with_initialize(
                TestCase {
                    test_messages: vec![request.clone().into()],
                    expected_err: "".to_string(),
                    expected_send: vec![Response::new_err(
                        req_id,
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {:?}", request),
                    )
                    .into()],
                    channel_works_ok: true,
                    is_ok: true,
                },
                true,
            );
        }

        #[test]
        fn some_notif_before_init_req() {
            let notification = communication::Notification {
                method: InitializedBuild::METHOD.to_string(),
                params: to_value(InitializedBuildParams::default()).unwrap(),
            };

            test_f_with_initialize(
                TestCase {
                    test_messages: vec![notification.into()],
                    expected_err: "".to_string(),
                    expected_send: vec![],
                    channel_works_ok: true,
                    is_ok: true,
                },
                true,
            );
        }

        #[test]
        fn exit_notif_before_init_req() {
            let notification = communication::Notification {
                method: ExitBuild::METHOD.to_string(),
                params: to_value(()).unwrap(),
            };
            let notification_msg = Message::from(notification);

            test_f(TestCase {
                test_messages: vec![notification_msg.clone()],
                expected_err: format!("expected initialize request, got {:?}", notification_msg),
                expected_send: vec![],
                channel_works_ok: true,
                is_ok: false,
            });
        }

        #[test]
        fn wrong_msg_before_init_req() {
            let response = communication::Response {
                id: RequestId::from(123),
                result: Some(to_value(CleanCacheResult::default()).unwrap()),
                error: None,
            };

            test_f(TestCase {
                test_messages: vec![response.clone().into()],
                expected_err: format!(
                    "expected initialize request, got {:?}",
                    Message::from(response)
                ),
                expected_send: vec![],
                channel_works_ok: true,
                is_ok: false,
            });
        }

        #[test]
        fn channel_err_before_init_req() {
            test_f(TestCase {
                test_messages: vec![],
                expected_err: format!("expected initialize request, got error: {}", RecvError {}),
                expected_send: vec![],
                channel_works_ok: false,
                is_ok: false,
            });
        }

        #[test]
        fn wrong_msg_before_init_notif() {
            let request = communication::Request {
                id: RequestId::from(123),
                method: Sources::METHOD.to_string(),
                params: to_value(SourcesParams::default()).unwrap(),
            };

            test_f_with_initialize(
                TestCase {
                    test_messages: vec![request.clone().into()],
                    expected_err: format!(
                        r#"expected initialized notification, got: {:?}"#,
                        Message::from(request)
                    ),
                    expected_send: vec![],
                    channel_works_ok: true,
                    is_ok: false,
                },
                false,
            );
        }

        #[test]
        fn channel_err_before_init_notif() {
            test_f_with_initialize(
                TestCase {
                    test_messages: vec![],
                    expected_err: format!(
                        "expected initialized notification, got error: {}",
                        RecvError {},
                    ),
                    expected_send: vec![],
                    channel_works_ok: false,
                    is_ok: false,
                },
                false,
            );
        }
    }
}
