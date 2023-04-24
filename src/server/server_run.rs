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
        use crossbeam_channel::RecvError;

        use crate::communication::{Connection, ErrorCode, Message, Response};
        use crate::server::config::Config;
        use crate::server::server_run::{create_initialize_result, initialize};
        use crate::server::Result;
        use crate::test_utils::{
            test_exit_notif, test_init_notif, test_init_params, test_init_req, test_init_resp,
            test_sources_req, test_sources_resp, TestCase,
        };

        struct InitTestCase {
            case: TestCase,
            msg_before_req: bool,
            add_req: bool,
            add_notif: bool,
        }

        fn initialize_order_test(mut test_case: InitTestCase) {
            let test_id = 234;

            let init_params = test_init_params();
            let config = Config::from_initialize_params(init_params.clone()).unwrap();

            let init_req = test_init_req(&init_params, test_id);
            let init_resp = test_init_resp(&create_initialize_result(&config), test_id);

            if test_case.add_req {
                test_case.case.expected_send.push(init_resp.into());
                if test_case.msg_before_req {
                    let mut temp = vec![init_req.into()];
                    temp.append(&mut test_case.case.test_messages);
                    test_case.case.test_messages = temp;
                } else {
                    test_case.case.test_messages.push(init_req.into());
                }
            }

            if test_case.add_notif {
                test_case.case.test_messages.push(test_init_notif().into());
            }

            test_case.case.func_to_test =
                |server: Connection| -> Result<()> { initialize(&server).map(|_| ()) };

            test_case.case.test();
        }

        #[test]
        fn proper_initialize() {
            initialize_order_test(InitTestCase {
                case: TestCase::default(),
                msg_before_req: false,
                add_req: true,
                add_notif: true,
            });
        }

        #[test]
        fn wrong_req_before_init_req() {
            let test_id = 123;
            let request = test_sources_req(test_id);

            initialize_order_test(InitTestCase {
                case: TestCase {
                    test_messages: vec![request.clone().into()],
                    expected_send: vec![Response::new_err(
                        test_id.into(),
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {:?}", request),
                    )
                    .into()],
                    ..TestCase::default()
                },
                msg_before_req: false,
                add_req: true,
                add_notif: true,
            });
        }

        #[test]
        fn some_notif_before_init_req() {
            initialize_order_test(InitTestCase {
                case: TestCase {
                    test_messages: vec![test_init_notif().into()],
                    ..TestCase::default()
                },
                msg_before_req: false,
                add_req: true,
                add_notif: true,
            });
        }

        #[test]
        fn exit_notif_before_init_req() {
            let notification_msg = Message::from(test_exit_notif());

            initialize_order_test(InitTestCase {
                case: TestCase {
                    test_messages: vec![notification_msg.clone()],
                    expected_err: format!(
                        "expected initialize request, got {:?}",
                        notification_msg
                    ),
                    is_ok: false,
                    ..TestCase::default()
                },
                msg_before_req: false,
                add_req: false,
                add_notif: false,
            });
        }

        #[test]
        fn wrong_msg_before_init_req() {
            let wrong_msg = test_sources_resp(123);

            initialize_order_test(InitTestCase {
                case: TestCase {
                    test_messages: vec![wrong_msg.clone().into()],
                    expected_err: format!(
                        "expected initialize request, got {:?}",
                        Message::from(wrong_msg)
                    ),
                    is_ok: false,
                    ..TestCase::default()
                },
                msg_before_req: false,
                add_req: false,
                add_notif: false,
            });
        }

        #[test]
        fn channel_err_before_init_req() {
            initialize_order_test(InitTestCase {
                case: TestCase {
                    expected_err: format!(
                        "expected initialize request, got error: {}",
                        RecvError {}
                    ),
                    channel_works_ok: false,
                    is_ok: false,
                    ..TestCase::default()
                },
                msg_before_req: false,
                add_req: false,
                add_notif: false,
            });
        }

        #[test]
        fn wrong_msg_before_init_notif() {
            let wrong_msg = test_sources_resp(123);

            initialize_order_test(InitTestCase {
                case: TestCase {
                    test_messages: vec![wrong_msg.clone().into()],
                    expected_err: format!(
                        r#"expected initialized notification, got: {:?}"#,
                        Message::from(wrong_msg)
                    ),
                    is_ok: false,
                    ..TestCase::default()
                },
                msg_before_req: true,
                add_req: true,
                add_notif: false,
            });
        }

        #[test]
        fn channel_err_before_init_notif() {
            initialize_order_test(InitTestCase {
                case: TestCase {
                    expected_err: format!(
                        "expected initialized notification, got error: {}",
                        RecvError {},
                    ),
                    channel_works_ok: false,
                    is_ok: false,
                    ..TestCase::default()
                },
                msg_before_req: false,
                add_req: true,
                add_notif: false,
            });
        }
    }
}
