#![warn(unused_lifetimes, semicolon_in_expressions_from_macros)]
#![allow(unused_variables)]

use std::{
    io,
    process::{ChildStderr, ChildStdout, Command, Stdio},
};

pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use command_group::{CommandGroup, GroupChild};
use crossbeam_channel::{never, Receiver, select, Sender, unbounded};
use serde::Deserialize;

use stdx::process::streaming_output;

use crate::bsp_types::notifications::{
    StatusCode, TaskFinishParams, TaskId, TaskProgressParams, TaskStartParams,
};
use crate::bsp_types::requests::{CreateCommand, Request};
use crate::bsp_types::notifications::{StatusCode, TaskFinishParams, TaskId, TaskProgressParams,
                                      TaskStartParams};
use crate::bsp_types::requests::CreateCommand;
use crate::bsp_types::OriginId;
use crate::communication::{RequestId, Response};
use crate::communication::Message as RPCMessage;
use crate::logger::log;
use crate::server::request_actor::Event::Cancel;

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
    ) -> RequestHandle
        where
            R: Request + 'static,
            R::Params: CreateCommand + Send,
    {
        let actor: RequestActor<R> = RequestActor::new(sender_to_main, req_id, params);
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();
        let thread = jod_thread::Builder::new()
            .spawn(move || actor.run(receiver_to_cancel))
            .expect("failed to spawn thread");
        RequestHandle { sender_to_cancel, _thread: thread }
    }

    #[allow(dead_code)]
    pub fn cancel(&self) {
        self.sender_to_cancel.send(Cancel).unwrap();
    }
}

#[derive(Debug)]
pub enum TaskNotification {
    Start(TaskStartParams),
    #[allow(dead_code)]
    Progress(TaskProgressParams),
    Finish(TaskFinishParams),
}

pub struct CargoMessage {}

pub struct RequestActor<R>
    where
        R: Request,
        R::Params: CreateCommand,
{
    sender: Box<dyn Fn(RPCMessage) + Send>,
    // config: CargoCommand,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo build/run/test` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<CargoHandle>,
    #[allow(dead_code)]
    req_id: RequestId,
    params: R::Params,
}

pub enum Event {
    Cancel,
    CargoEvent(Option<CargoMessage>),
}

impl<R> RequestActor<R>
    where
        R: Request,
        R::Params: CreateCommand,
{
    pub fn new(
        sender: Box<dyn Fn(RPCMessage) + Send>,
        req_id: RequestId,
        params: R::Params,
    ) -> RequestActor<R> {
        log("Spawning a new request actor");
        RequestActor { sender, cargo_handle: None, req_id, params }
    }

    fn report_task_start(&self, task_id: TaskId) {
        // TODO improve this
        self.send_notification(TaskNotification::Start(TaskStartParams {
            task_id,
            event_time: None,
            message: None,
            data: None,
        }));
    }

    #[allow(dead_code)]
    fn report_task_progress(&self, task_id: TaskId, message: Option<String>) {
        // TODO improve this
        self.send_notification(TaskNotification::Progress(TaskProgressParams {
            task_id,
            event_time: None,
            message,
            total: None,
            progress: None,
            data: None,
            unit: None,
        }));
    }

    fn report_task_finish(&self, task_id: TaskId, status_code: StatusCode) {
        // TODO improve this
        self.send_notification(TaskNotification::Finish(TaskFinishParams {
            task_id,
            event_time: None,
            message: None,
            status: status_code,
            data: None,
        }));
    }

    fn send_notification(&self, progress: TaskNotification) {
        todo!()
    }

    fn next_event(&self, inbox: &Receiver<Event>) -> Option<Event> {
        let cargo_chan = self.cargo_handle.as_ref().map(|cargo| &cargo.receiver);
        select! {
            recv(inbox) -> msg => msg.ok(),
            recv(cargo_chan.unwrap_or(&never())) -> msg => Some(Event::CargoEvent(msg.ok())),
        }
    }

    pub fn run(mut self, cancel_receiver: Receiver<Event>) {
        let command = self.params.create_command();
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => {
                self.cargo_handle = Some(cargo_handle);
                self.report_task_start(TaskId { id: self.params.origin_id().unwrap(), parents: None });
            }
            Err(err) => {
                todo!()
            }
        }
        while let Some(event) = self.next_event(&cancel_receiver) {
            match event {
                Cancel => {
                    self.cancel_process();
                    return;
                }
                Event::CargoEvent(None) => {
                    // Watcher finished
                    let cargo_handle = self.cargo_handle.take().unwrap();
                    let res = cargo_handle.join();
                    if res.is_err() {
                        self.report_task_finish(TaskId { id: self.params.origin_id().unwrap(), parents: None }, StatusCode::Error);
                        todo!()
                    }
                    self.report_task_finish(TaskId { id: self.params.origin_id().unwrap(), parents: None }, StatusCode::Ok);
                }
                Event::CargoEvent(Some(message)) => {
                    // handle information and create reponse/notification based on that
                    let resp = RPCMessage::Response(Response {
                        id: RequestId::from(0),
                        result: None,
                        error: None,
                    });
                    self.send(resp);
                    // shouldn't we break the loop after sending response message?
                }
            }
        }
    }

    fn cancel_process(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            self.report_task_start(TaskId {
                id: OriginId::from("TODO".to_string()),
                parents: Some(vec![self.params.origin_id().unwrap()]),
            });
            cargo_handle.cancel();
            self.report_task_finish(TaskId {
                id: OriginId::from("TODO".to_string()),
                parents: Some(vec![self.params.origin_id().unwrap()]),
            }, StatusCode::Cancelled, );
            self.report_task_finish(TaskId { id: self.params.origin_id().unwrap(), parents: None },
                                    StatusCode::Cancelled);
            // TODO
        } else {
            todo!()
        }
    }

    fn send(&self, msg: RPCMessage) { (self.sender)(msg); }
}

struct JodChild(GroupChild);

struct CargoHandle {
    /// The handle to the actual cargo process. As we cannot cancel directly from with
    /// a read syscall dropping and therefore terminating the process is our best option.
    child: JodChild,
    thread: jod_thread::JoinHandle<io::Result<(bool, String)>>,
    receiver: Receiver<CargoMessage>,
}

impl CargoHandle {
    fn spawn(mut command: Command) -> io::Result<CargoHandle> {
        command.stdout(Stdio::piped()).stderr(Stdio::piped()).stdin(Stdio::null());
        let mut child = command.group_spawn().map(JodChild)?;

        let stdout = child.0.inner().stdout.take().unwrap();
        let stderr = child.0.inner().stderr.take().unwrap();

        let (sender, receiver) = unbounded();
        let actor = CargoActor::new(sender, stdout, stderr);
        let thread = jod_thread::Builder::new()
            .name("CargoHandle".to_owned())
            .spawn(move || actor.run())
            .expect("failed to spawn thread");
        Ok(CargoHandle { child, thread, receiver })
    }

    fn cancel(mut self) {
        let _ = self.child.0.kill();
        let _ = self.child.0.wait();
    }

    fn join(mut self) -> io::Result<()> {
        let _ = self.child.0.kill();
        let exit_status = self.child.0.wait()?;
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

struct CargoActor {
    sender: Sender<CargoMessage>,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl CargoActor {
    fn new(sender: Sender<CargoMessage>, stdout: ChildStdout, stderr: ChildStderr) -> CargoActor {
        CargoActor { sender, stdout, stderr }
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

        let mut error = String::new();
        let mut read_at_least_one_message = false;
        let output = streaming_output(
            self.stdout,
            self.stderr,
            &mut |line| {
                read_at_least_one_message = true;

                // Try to deserialize a message from Cargo or Rustc.
                let mut deserializer = serde_json::Deserializer::from_str(line);
                deserializer.disable_recursion_limit();
                if let Ok(message) = JsonMessage::deserialize(&mut deserializer) {
                    self.sender.send(CargoMessage {}).expect("TODO: panic message");
                }
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

#[derive(Deserialize)]
#[serde(untagged)]
enum JsonMessage {
    Cargo(cargo_metadata::Message),
    Rustc(Diagnostic),
}
