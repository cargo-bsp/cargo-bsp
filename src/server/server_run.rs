use bsp_server::Connection;
use log::info;

use crate::bsp_types::requests::{InitializeBuildParams, InitializeBuildResult};
use crate::server;
use crate::server::caps::server_capabilities;
use crate::server::config::Config;
use crate::server::{from_json, Result};

pub fn run_server() -> Result<()> {
    info!("server will start");

    let (connection, io_threads) = Connection::bsp_stdio();

    let config = initialize(&connection)?;

    server::main_loop(config, connection)?;

    io_threads.join()?;
    info!("server did shut down");
    Ok(())
}

fn initialize(connection: &Connection) -> Result<Config> {
    let (initialize_id, initialize_params) = connection.bsp_initialize_start()?;
    let initialize_params =
        from_json::<InitializeBuildParams>("InitializeParams", &initialize_params)?;

    let config = Config::from_initialize_params(initialize_params)?;
    let initialize_result = create_initialize_result(&config);

    connection.bsp_initialize_finish(
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
        use bsp_server::{Connection, ErrorCode, Message, Response};
        use crossbeam_channel::RecvError;

        use crate::server::config::Config;
        use crate::server::server_run::{create_initialize_result, initialize};
        use crate::server::Result;
        use crate::test_utils::{
            test_exit_notif, test_init_notif, test_init_params, test_init_req, test_init_resp,
            test_sources_req, test_sources_resp, TestCase,
        };

        struct InitTestCase {
            case: TestCase,
            is_init_req_first: bool,
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
                test_case.case.expected_recv.push(init_resp.into());
                if test_case.is_init_req_first {
                    test_case.case.to_send.push(init_req.into());
                } else {
                    test_case.case.to_send.insert(0, init_req.into());
                }
            }

            if test_case.add_notif {
                test_case.case.to_send.push(test_init_notif().into());
            }

            test_case.case.func_to_test =
                |server: Connection| -> Result<()> { initialize(&server).map(|_| ()) };

            test_case.case.test();
        }

        #[test]
        fn proper_initialize() {
            initialize_order_test(InitTestCase {
                case: TestCase::new(true, true),
                is_init_req_first: true,
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
                    to_send: vec![request.clone().into()],
                    expected_recv: vec![Response::new_err(
                        test_id.into(),
                        ErrorCode::ServerNotInitialized as i32,
                        format!("expected initialize request, got {:?}", request),
                    )
                    .into()],
                    ..TestCase::new(true, true)
                },
                is_init_req_first: true,
                add_req: true,
                add_notif: true,
            });
        }

        #[test]
        fn some_notif_before_init_req() {
            initialize_order_test(InitTestCase {
                case: TestCase {
                    to_send: vec![test_init_notif().into()],
                    ..TestCase::new(true, true)
                },
                is_init_req_first: true,
                add_req: true,
                add_notif: true,
            });
        }

        #[test]
        fn exit_notif_before_init_req() {
            let notification_msg = Message::from(test_exit_notif());

            initialize_order_test(InitTestCase {
                case: TestCase {
                    to_send: vec![notification_msg.clone()],
                    expected_err: format!(
                        "expected initialize request, got {:?}",
                        notification_msg
                    ),
                    ..TestCase::new(true, false)
                },
                is_init_req_first: true,
                add_req: false,
                add_notif: false,
            });
        }

        #[test]
        fn wrong_msg_before_init_req() {
            let wrong_msg = test_sources_resp(123);

            initialize_order_test(InitTestCase {
                case: TestCase {
                    to_send: vec![wrong_msg.clone().into()],
                    expected_err: format!(
                        "expected initialize request, got {:?}",
                        Message::from(wrong_msg)
                    ),
                    ..TestCase::new(true, false)
                },
                is_init_req_first: true,
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
                    ..TestCase::new(false, false)
                },
                is_init_req_first: true,
                add_req: false,
                add_notif: false,
            });
        }

        #[test]
        fn wrong_msg_before_init_notif() {
            let wrong_msg = test_sources_resp(123);

            initialize_order_test(InitTestCase {
                case: TestCase {
                    to_send: vec![wrong_msg.clone().into()],
                    expected_err: format!(
                        r#"expected initialized notification, got: {:?}"#,
                        Message::from(wrong_msg)
                    ),
                    ..TestCase::new(true, false)
                },
                is_init_req_first: false,
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
                    ..TestCase::new(false, false)
                },
                is_init_req_first: true,
                add_req: true,
                add_notif: false,
            });
        }
    }
}
