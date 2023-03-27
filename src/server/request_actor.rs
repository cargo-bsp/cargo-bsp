#![warn(unused_lifetimes, semicolon_in_expressions_from_macros)]
#![allow(unused_variables)]

use std::path::{Path, PathBuf};
use std::{
    io,
    process::{ChildStderr, ChildStdout, Command, Stdio},
};
use crate::bsp_types::BuildTargetIdentifier;
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use cargo_metadata::Message as CargoMessage;
use command_group::{CommandGroup, GroupChild};
use crossbeam_channel::{never, select, unbounded, Receiver, Sender};
use lsp_types::DiagnosticSeverity;
use paths::AbsPath;
use serde::Deserialize;
use serde_json::to_value;
use stdx::process::streaming_output;
use mockall::*;

use crate::bsp_types::notifications::{
    LogMessage, LogMessageParams, MessageType, Notification as NotificationTrait,
    PublishDiagnostics, PublishDiagnosticsParams, StatusCode, TaskFinish, TaskFinishParams, TaskId,
    TaskProgress, TaskProgressParams, TaskStart, TaskStartParams,
};
use crate::bsp_types::requests::{CreateCommand, Request};
use crate::communication::{Message as RPCMessage, Notification};
use crate::communication::{RequestId, Response};
use crate::logger::log;

pub enum Event {
    Cancel,
    CargoEvent(CargoMessage),
    CargoFinish,
}

#[derive(Debug)]
pub struct RequestHandle {
    #[allow(dead_code)]
    sender_to_cancel: Sender<Event>,
    _thread: jod_thread::JoinHandle,
}

impl RequestHandle {
    pub fn spawn<R>(
        sender_to_main: Box<dyn Fn(RPCMessage) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> RequestHandle
        where
            R: Request + 'static,
            R::Params: CreateCommand + Send,
    {
        let mut actor: RequestActor<R, CargoHandle> = RequestActor::new(sender_to_main, req_id, params, root_path);
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();
        let thread = jod_thread::Builder::new()
            .spawn(move || {
                match actor.spawn_handle() {
                    Ok(cargo_handle) => actor.run(receiver_to_cancel, cargo_handle),
                    Err(err) => {
                        todo!()
                    }
                }
            })
            .expect("failed to spawn thread");
        RequestHandle {
            sender_to_cancel,
            _thread: thread,
        }
    }

    #[allow(dead_code)]
    pub fn cancel(&self) {
        self.sender_to_cancel.send(Event::Cancel).unwrap();
    }
}

pub struct RequestActor<R, C>
    where
        R: Request,
        R::Params: CreateCommand,
        C: CargoHandleTrait<CargoMessage>,
{
    sender: Box<dyn Fn(RPCMessage) + Send>,
    // config: CargoCommand,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<C>,
    #[allow(dead_code)]
    req_id: RequestId,
    params: R::Params,
    root_path: PathBuf,
}

impl<R, C> RequestActor<R, C>
    where
        R: Request,
        R::Params: CreateCommand,
        C: CargoHandleTrait<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(RPCMessage) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> RequestActor<R, C> {
        log("Spawning a new request actor");
        RequestActor {
            sender,
            cargo_handle: None,
            req_id,
            params,
            root_path: root_path.to_path_buf(),
        }
    }

    fn report_task_start(&self, task_id: TaskId) {
        // TODO improve this
        self.send_notification::<TaskStart>(TaskStartParams {
            task_id,
            event_time: None,
            message: None,
            data: None,
        });
    }

    fn report_task_progress(&self, task_id: TaskId, message: Option<String>) {
        // TODO improve this
        self.send_notification::<TaskProgress>(TaskProgressParams {
            task_id,
            event_time: None,
            message,
            total: None,
            progress: None,
            data: None,
            unit: None,
        });
    }

    fn report_task_finish(&self, task_id: TaskId, status_code: StatusCode) {
        // TODO improve this
        self.send_notification::<TaskFinish>(TaskFinishParams {
            task_id,
            event_time: None,
            message: None,
            status: status_code,
            data: None,
        });
    }

    fn send_notification<T>(&self, progress: T::Params)
        where
            T: NotificationTrait,
    {
        self.send(
            Notification {
                method: T::METHOD.to_string(),
                params: to_value(progress).unwrap(),
            }
                .into(),
        );
    }

    fn next_event(&self, inbox: &Receiver<Event>) -> Option<Event> {
        let cargo_chan = self.cargo_handle.as_ref().map(|cargo| cargo.receiver());
        select! {
            recv(inbox) -> msg => msg.ok(),
            recv(cargo_chan.unwrap_or(&never())) -> msg => match msg {
                Ok(msg) => Some(Event::CargoEvent(msg)),
                Err(_) => Some(Event::CargoFinish),
            }
        }
    }

    pub fn spawn_handle(&mut self) -> Result<CargoHandle, String> {
        let command = self.params.create_command(self.root_path.clone());
        log(format!("Created command: {:?}", command).as_str());
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => Ok(cargo_handle),
            Err(err) => { todo!() }
        }
    }

    pub fn run(mut self, cancel_receiver: Receiver<Event>, cargo_handle: C) {
        self.report_task_start(TaskId {
            id: self.params.origin_id().unwrap(),
            parents: vec![],
        });

        self.cargo_handle = Some(cargo_handle);
        // Errors and warnings for compile report.
        let mut errors = 0;
        let mut warnings = 0;

        while let Some(event) = self.next_event(&cancel_receiver) {
            match event {
                Event::Cancel => {
                    self.cancel_process();
                    return;
                }
                Event::CargoFinish => {
                    // Watcher finished
                    let cargo_handle = self.cargo_handle.take().unwrap();
                    let res = cargo_handle.join();
                    #[allow(unused_mut)]
                        let mut resp = RPCMessage::Response(Response {
                        id: self.req_id.clone(),
                        result: None,
                        error: None,
                    });
                    let status_code = match res {
                        Ok(_) => StatusCode::Ok,
                        Err(err) => StatusCode::Error,
                    };
                    self.report_task_finish(
                        TaskId {
                            id: self.params.origin_id().unwrap(),
                            parents: vec![],
                        },
                        status_code,
                    );
                    self.send(resp);
                    return;
                }
                Event::CargoEvent(message) => {
                    // handle information and create notification based on that
                    match message {
                        CargoMessage::CompilerArtifact(msg) => {
                            self.report_task_progress(
                                TaskId {
                                    // TODO generate id when there is no origin_id
                                    id: self.params.origin_id().unwrap(),
                                    parents: vec![],
                                },
                                serde_json::to_string(&msg).ok(),
                            );
                        }
                        CargoMessage::CompilerMessage(msg) => {
                            let diagnostics = PublishDiagnosticsParams::from(
                                &msg.message,
                                self.params.origin_id(),
                                // TODO change to actual BuildTargetIdentifier
                                &BuildTargetIdentifier {
                                    uri: "".to_string(),
                                },
                                AbsPath::assert(&self.root_path),
                            );
                            diagnostics.into_iter().for_each(|diagnostic| {
                                // Count errors and warnings.
                                diagnostic.diagnostics.iter().for_each(|d| {
                                    if let Some(severity) = d.severity {
                                        match severity {
                                            DiagnosticSeverity::ERROR => errors += 1,
                                            DiagnosticSeverity::WARNING => warnings += 1,
                                            _ => (),
                                        }
                                    }
                                });
                                self.send_notification::<PublishDiagnostics>(diagnostic)
                            });
                        }
                        CargoMessage::BuildScriptExecuted(msg) => {
                            self.report_task_progress(
                                TaskId {
                                    // TODO generate id when there is no origin_id
                                    id: self.params.origin_id().unwrap(),
                                    parents: vec![],
                                },
                                serde_json::to_string(&msg).ok(),
                            );
                        }
                        CargoMessage::BuildFinished(_) => {
                            // TODO generate compile report
                        }
                        CargoMessage::TextLine(msg) => {
                            self.send_notification::<LogMessage>(LogMessageParams {
                                message_type: MessageType::Log,
                                task: self.params.origin_id().map(|id| TaskId {
                                    id,
                                    parents: vec![],
                                }),
                                origin_id: self.params.origin_id(),
                                message: msg,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn cancel_process(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            self.report_task_start(TaskId {
                id: "TODO Start cancel".to_string(),
                parents: vec![self.params.origin_id().unwrap()],
            });
            cargo_handle.cancel();
            self.report_task_finish(
                TaskId {
                    id: "TODO Finish cancel".to_string(),
                    parents: vec![self.params.origin_id().unwrap()],
                },
                StatusCode::Ok,
            );
            self.report_task_finish(
                TaskId {
                    id: self.params.origin_id().unwrap(),
                    parents: vec![],
                },
                StatusCode::Cancelled,
            );
            // TODO
        } else {
            todo!()
        }
    }

    fn send(&self, msg: RPCMessage) {
        (self.sender)(msg);
    }
}


#[automock]
pub trait CargoHandleTrait<T> {
    fn receiver(&self) -> &Receiver<T>;

    fn cancel(self);

    fn join(self) -> io::Result<()>;
}

pub struct CargoHandle {
    /// The handle to the actual cargo process. As we cannot cancel directly from with
    /// a read syscall dropping and therefore terminating the process is our best option.
    child: GroupChild,
    thread: jod_thread::JoinHandle<io::Result<(bool, String)>>,
    receiver: Receiver<CargoMessage>,
}

impl CargoHandleTrait<CargoMessage> for CargoHandle {
    fn receiver(&self) -> &Receiver<CargoMessage> {
        &self.receiver
    }

    fn cancel(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn join(mut self) -> io::Result<()> {
        let _ = self.child.kill();
        let exit_status = self.child.wait()?;
        let (read_at_least_one_message, error) = self.thread.join()?;
        if read_at_least_one_message || exit_status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!(
                "Cargo watcher failed, the command produced no valid metadata (exit code: {:?}):\n{}",
                exit_status, error
            )))
        }
    }
}

impl CargoHandle {
    fn spawn(mut command: Command) -> io::Result<CargoHandle> {
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        let mut child = command.group_spawn()?;

        let stdout = child.inner().stdout.take().unwrap();
        let stderr = child.inner().stderr.take().unwrap();

        let (sender, receiver) = unbounded();
        let actor = CargoActor::new(sender, stdout, stderr);
        let thread = jod_thread::Builder::new()
            .name("CargoHandle".to_owned())
            .spawn(move || actor.run())
            .expect("failed to spawn thread");
        Ok(CargoHandle {
            child,
            thread,
            receiver,
        })
    }
}

struct CargoActor {
    sender: Sender<CargoMessage>,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl CargoActor {
    fn new(sender: Sender<CargoMessage>, stdout: ChildStdout, stderr: ChildStderr) -> CargoActor {
        CargoActor {
            sender,
            stdout,
            stderr,
        }
    }

    fn run(self) -> io::Result<(bool, String)> {
        // We manually read a line at a time, instead of using serde's
        // stream deserializers, because the deserializer cannot recover
        // from an error, resulting in it getting stuck, because we try to
        // be resilient against failures.
        //
        // Because cargo only outputs one JSON object per line, we can
        // simply skip a line if it doesn't parse, which just ignores any
        // erroneous output.
        //
        // We return bool that indicates whether we read at least one message and a string that
        // contains the error output.

        let mut error = String::new();
        let mut read_at_least_one_message = false;
        let output = streaming_output(
            self.stdout,
            self.stderr,
            &mut |line| {
                read_at_least_one_message = true;

                // Try to deserialize a message from Cargo.
                let mut deserializer = serde_json::Deserializer::from_str(line);
                deserializer.disable_recursion_limit();
                match CargoMessage::deserialize(&mut deserializer) {
                    Ok(message) => {
                        self.sender.send(message).expect("TODO: panic message");
                    }
                    Err(e) => {
                        // todo!("Log that we couldn't parse a message: {:?}", line")
                    }
                };
            },
            &mut |line| {
                error.push_str(line);
                error.push('\n');
            },
        );
        match output {
            Ok(_) => Ok((read_at_least_one_message, error)),
            Err(e) => Err(io::Error::new(e.kind(), format!("{:?}: {}", e, error))),
        }
    }
}

#[cfg(test)]
pub mod request_actor_tests {
    use std::marker::PhantomData;
    use super::*;
    use crate::bsp_types::requests::{Compile, CompileParams, Run, RunParams, Test, TestParams};
    use crate::bsp_types::notifications::{TaskStart, TaskStartParams, TaskFinish, TaskFinishParams, TaskProgress, TaskProgressParams, TaskId, StatusCode, TestDataWithKind, CompileTaskData};
    use crate::communication::Message;
    use crate::communication::Notification;

    // trait GetParams {
    //     type T;
    //
    //     fn get_params() -> Self::T;
    // }
    //
    // impl GetParams for Compile {
    //     type T = CompileParams;
    //
    //     fn get_params() -> Self::T {
    //         CompileParams {
    //             targets: vec!["test_target".into()],
    //             origin_id: Some("test_origin".into()),
    //             arguments: Some(vec!["test_arguments".into()]),
    //         }
    //     }
    // }
    //
    // impl GetParams for Run {
    //     type T = RunParams;
    //
    //     fn get_params() -> Self::T {
    //         RunParams {
    //             target: "test_target".into(),
    //             origin_id: Some("test_origin".into()),
    //             arguments: Some(vec!["test_arguments".into()]),
    //             data_kind: Some("test_data_kind".into()),
    //             data: Some("test_data".into()),
    //         }
    //     }
    // }
    //
    // impl GetParams for Test {
    //     type T = TestParams;
    //
    //     fn get_params() -> Self::T {
    //         TestParams {
    //             targets: vec!["test_target".into()],
    //             origin_id: Some("test_origin".into()),
    //             arguments: Some(vec!["test_arguments".into()]),
    //             data_kind: Some("test_data_kind".into()),
    //             data: Some("test_data".into()),
    //         }
    //     }
    // }

    fn get_test_actor(sender_to_main: Sender<RPCMessage>) -> RequestActor<Test, MockCargoHandleTrait<CargoMessage>> {
        RequestActor::new(
            Box::new(move |msg| sender_to_main.send(msg).unwrap()),
            "test_req_id".into(),
            TestParams {
                targets: vec!["test_target".into()],
                origin_id: Some("test_origin_id".into()),
                arguments: Some(vec!["test_arguments".into()]),
                data_kind: Some("test_data_kind".into()),
                data: Some("test_data".into()),
            },
            Path::new("test_root_path"),
        )
    }

    fn get_compile_actor(sender_to_main: Sender<RPCMessage>) -> RequestActor<Compile, MockCargoHandleTrait<CargoMessage>> {
        RequestActor::new(
            Box::new(move |msg| sender_to_main.send(msg).unwrap()),
            "test_req_id".into(),
            CompileParams {
                targets: vec!["test_target".into()],
                origin_id: Some("test_origin_id".into()),
                arguments: Some(vec!["test_arguments".into()]),
            },
            Path::new("test_root_path"),
        )
    }

    fn get_run_actor(sender_to_main: Sender<RPCMessage>) -> RequestActor<Run, MockCargoHandleTrait<CargoMessage>> {
        RequestActor::new(
            Box::new(move |msg| sender_to_main.send(msg).unwrap()),
            "test_req_id".into(),
            RunParams {
                target: "test_target".into(),
                origin_id: Some("test_origin_id".into()),
                arguments: Some(vec!["test_arguments".into()]),
                data_kind: Some("test_data_kind".into()),
                data: Some("test_data".into()),
            },
            Path::new("test_root_path"),
        )
    }

    #[test]
    fn simple_compile() {
        let (sender_to_main, receiver_to_main) = unbounded::<RPCMessage>();
        let (sender_from_cargo, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();

        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);

        let req_actor = get_compile_actor(sender_to_main);


        // sender_to_cancel.send(Event::Cancel).unwrap();
        //
        // let proper_notif_start_task = Notification::new(
        //     TaskStart::METHOD.to_string(),
        //     TaskStartParams {
        //         task_id: TaskId { id: "test_origin_id" },
        //     },
        // );
        // let proper_notif_finish_task = Notification::new(
        //     TaskFinish::METHOD.to_string(),
        //     TaskFinishParams {
        //         task_id: TaskId { id: "test_origin_id" },
        //         status: StatusCode::Ok,
        //     },
        // );
        //
        // let proper_notif_progress_task = Notification::new(
        //     TaskProgress::METHOD.to_string(),
        //     TaskProgressParams {
        //         task_id: TaskId { id: "test_origin_id" },
        //         message: "test_message".into(),
        //         percentage: Some(0.5),
        //     },
        // );
        //
        // let proper_notif_progress_task2 = Notification::new(
        //     TaskProgress::METHOD.to_string(),
        //     TaskProgressParams {
        //         task_id: TaskId { id: "test_origin_id" },
        //         message: "test_message2".into(),
        //         percentage: Some(0.5),
        //     },
        // );
        //
        // let proper_notif_progress_task3 = Notification::new(
        //     TaskProgress::METHOD.to_string(),
        //     TaskProgressParams {
        //         task_id: TaskId { id: "test_origin_id" },
        //         message: "test_message3".into(),
        //         percentage: Some(0.5),
        //     },
        // );
        //
        // let proper_notif_progress_task4 = Notification::new(
        //     TaskProgress::METHOD.to_string(),
        //     TaskProgressParams {
        //         task_id: TaskId { id: "test_origin_id" },
        //         message: "test_message4".into(),
        //         percentage: Some(0.5),
        //     },
        // );
        //
        // let proper_notif_progress_task5 = Notification::new(
        //     TaskProgress::METHOD.to_string
    }

    #[test]
    fn simple_cancel() {
        let (sender_to_main, receiver_to_main) = unbounded::<RPCMessage>();
        let (sender_from_cargo, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();

        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);
        mock_cargo_handle
            .expect_cancel()
            .return_const(());

        let req_actor = get_compile_actor(sender_to_main);
        sender_to_cancel.send(Event::Cancel).unwrap();

        let proper_notif_start_task = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId { id: "test_origin_id".to_string(), parents: vec![] },
                event_time: None,
                message: None,
                data: Some(TestDataWithKind::CompileTask(CompileTaskData {
                    target: "test_target".into(),
                })),
            },
        );

        let proper_notif_start_cancel = Notification::new(
            TaskStart::METHOD.to_string(),
            TaskStartParams {
                task_id: TaskId { id: "TODO Start cancel".to_string(), parents: vec!["test_origin_id".to_string()] },
                event_time: None,
                message: None,
                data: None,
            },
        );
        let proper_notif_finish_cancel = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId { id: "TODO Finish cancel".to_string(), parents: vec!["test_origin_id".to_string()] },
                event_time: None,
                message: None,
                data: None,
                status: StatusCode::Ok,
            },
        );
        let proper_notif_finish_task = Notification::new(
            TaskFinish::METHOD.to_string(),
            TaskFinishParams {
                task_id: TaskId { id: "test_origin_id".to_string(), parents: vec![] },
                event_time: None,
                message: None,
                data: None,
                status: StatusCode::Cancelled,
            },
        );

        req_actor.run(receiver_to_cancel, mock_cargo_handle);

        assert_eq!(receiver_to_main.recv().unwrap(), Message::Notification(proper_notif_start_task));
        assert_eq!(receiver_to_main.recv().unwrap(), Message::Notification(proper_notif_start_cancel));
        assert_eq!(receiver_to_main.recv().unwrap(), Message::Notification(proper_notif_finish_cancel));
        assert_eq!(receiver_to_main.recv().unwrap(), Message::Notification(proper_notif_finish_task));
    }

    #[test]
    fn simple_compile2() {
        let (sender_to_main, receiver_to_main) = unbounded::<RPCMessage>();
        let (sender_from_cargo, receiver_from_cargo) = unbounded::<CargoMessage>();
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();
        let mut mock_cargo_handle = MockCargoHandleTrait::new();
        mock_cargo_handle
            .expect_receiver()
            .return_const(receiver_from_cargo);
        mock_cargo_handle
            .expect_cancel()
            .return_const(());
        let req_actor = get_test_actor(sender_to_main);

        // let thread = jod_thread::Builder::new()
        //     .spawn(move || { req_actor.run(receiver_to_cancel, mock_cargo_handle) })
        //     .expect("failed to spawn thread");
        let compiler_artifact_example = r#"{"reason":"compiler-artifact","package_id":"proc-macro2 1.0.51 (registry+https://github.com/rust-lang/crates.io-index)","manifest_path":"/home/patryk/.cargo/registry/src/github.com-1ecc6299db9ec823/proc-macro2-1.0.51/Cargo.toml","target":{"kind":["custom-build"],"crate_types":["bin"],"name":"build-script-build","src_path":"/home/patryk/.cargo/registry/src/github.com-1ecc6299db9ec823/proc-macro2-1.0.51/build.rs","edition":"2018","doc":false,"doctest":false,"test":false},"profile":{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false},"features":["default","proc-macro"],"filenames":["/home/patryk/bsp/RustHelloWorld/target/debug/build/proc-macro2-bb5f134a0bb81455/build-script-build"],"executable":null,"fresh":true}"#;
        sender_from_cargo.send(CargoMessage::CompilerArtifact(serde_json::from_str(compiler_artifact_example).unwrap())).unwrap();

        sender_to_cancel.send(Event::Cancel).unwrap();
        req_actor.run(receiver_to_cancel, mock_cargo_handle);

        while let Ok(msg) = receiver_to_main.try_recv() {
            log(format!("{:#?}", msg).as_str());
        }

        log("done");
    }
}

#[cfg(test)]
pub mod integration_tests {
    use super::*;
    use crate::bsp_types::requests::{Compile, CompileParams, Run, RunParams, Test, TestParams};

    #[test]
    fn compile_request_handle() {
        let params = CompileParams {
            targets: vec!["test_data".into()],
            origin_id: Some("test".to_string()),
            arguments: None,
        };
        let (sender, receiver) = unbounded();

        let handle = RequestHandle::spawn::<Compile>(
            Box::new(move |msg| sender.send(msg).unwrap()),
            1.into(),
            params,
            std::env::current_dir()
                .unwrap()
                .join("src/server/test_data")
                .as_path(),
        );
        while let Some(msg) = receiver.recv().ok() {
            println!("{:#?}", msg);
        }
    }

    #[test]
    fn test_request_handle() {
        let params = TestParams {
            targets: vec![],
            origin_id: Some("test".to_string()),
            arguments: None,
            data_kind: None,
            data: None,
        };
        let (sender, receiver) = unbounded();

        let handle = RequestHandle::spawn::<Test>(
            Box::new(move |msg| sender.send(msg).unwrap()),
            1.into(),
            params,
            std::env::current_dir()
                .unwrap()
                .join("src/server/test_data")
                .as_path(),
        );
        while let Some(msg) = receiver.recv().ok() {
            println!("{:#?}", msg);
        }
    }

    #[test]
    fn run_request_handle() {
        let params = RunParams {
            target: "test_data".into(),
            origin_id: Some("test".to_string()),
            arguments: None,
            data_kind: None,
            data: None,
        };
        let (sender, receiver) = unbounded();

        let handle = RequestHandle::spawn::<Run>(
            Box::new(move |msg| sender.send(msg).unwrap()),
            1.into(),
            params,
            std::env::current_dir()
                .unwrap()
                .join("src/server/test_data")
                .as_path(),
        );
        while let Some(msg) = receiver.recv().ok() {
            println!("{:#?}", msg);
        }
    }

    #[test]
    fn test_request_handle_cancel() {
        let (sender, receiver) = unbounded();
        let handle = RequestHandle::spawn::<Test>(
            Box::new(move |msg| sender.send(msg).unwrap()),
            1.into(),
            TestParams::default(),
            std::env::current_dir().unwrap().as_path(),
        );
        handle.cancel();
    }
}
