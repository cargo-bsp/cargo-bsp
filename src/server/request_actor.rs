//! Flycheck provides the functionality needed to run `cargo check` or
//! another compatible command (f.x. clippy) in a background thread and provide
//! LSP diagnostics based on the output of the command.

#![warn(rust_2018_idioms, unused_lifetimes, semicolon_in_expressions_from_macros)]

use std::{
    fmt, io,
    process::{ChildStderr, ChildStdout, Command, Stdio},
    time::Duration,
};

use command_group::{CommandGroup, GroupChild};
use crossbeam_channel::{never, select, unbounded, Receiver, Sender};
use paths::AbsPathBuf;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use stdx::process::streaming_output;

use crate::logger::log;

pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::bsp_types::notifications::{TaskFinishParams, TaskId, TaskProgressParams, TaskStartParams};
use crate::bsp_types::requests::{CompileParams, DebugSessionParams, RunParams, TestParams};
use crate::communication::{Message, Notification, Request, RequestId, Response};
use crate::server::request_actor::Event::Cancel;
use crate::communication::Message as RPCMessage;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum InvocationStrategy {
    Once,
    #[default]
    PerWorkspace,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum InvocationLocation {
    Root(AbsPathBuf),
    #[default]
    Workspace,
}

#[derive(Debug)]
pub enum CargoCommand {
    Compile(CompileParams),
    Debug(DebugSessionParams),
    Run(RunParams),
    Test(TestParams),
}

#[derive(Debug)]
pub struct RequestHandle {
    // XXX: drop order is significant
    sender: Sender<Event>,
    _thread: jod_thread::JoinHandle,
    id: usize,
}

impl RequestHandle {
    pub fn spawn(
        id: usize,
        sender: Box<dyn Fn(RPCMessage) + Send>,
        config: CargoCommand,
        workspace_root: AbsPathBuf,
        req: Request,
    ) -> RequestHandle {
        let actor = RequestActor::new(id, sender, config, workspace_root, req);
        let (sender, receiver) = unbounded::<Event>();
        let thread = jod_thread::Builder::new()
            .spawn(move || actor.run(receiver))
            .expect("failed to spawn thread");
        RequestHandle { id, sender, _thread: thread }
    }

    /// Stop this cargo check worker.
    pub fn cancel(&self) {
        self.sender.send(Cancel).unwrap();
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Debug)]
pub enum TaskNotification {
    Start(TaskStartParams),
    Progress(TaskProgressParams),
    Finish(TaskFinishParams),
}

pub struct CargoMessage {}

/// A [`RequestActor`] is a single check instance of a workspace.
pub struct RequestActor {
    /// The workspace id of this flycheck instance.
    id: usize,
    sender: Box<dyn Fn(RPCMessage) + Send>,
    config: CargoCommand,
    /// Either the workspace root of the workspace we are flychecking,
    /// or the project root of the project.
    root: AbsPathBuf,
    /// CargoHandle exists to wrap around the communication needed to be able to
    /// run `cargo check` without blocking. Currently the Rust standard library
    /// doesn't provide a way to read sub-process output without blocking, so we
    /// have to wrap sub-processes output handling in a thread and pass messages
    /// back over a channel.
    cargo_handle: Option<CargoHandle>,
    req: Request,
    task_id: TaskId, // we need to set it, maybe based on request
}

enum Event {
    Cancel,
    CargoEvent(Option<CargoMessage>),
}

impl RequestActor {
    pub fn new(
        id: usize,
        sender: Box<dyn Fn(RPCMessage) + Send>,
        config: CargoCommand,
        workspace_root: AbsPathBuf,
        req: Request,
    ) -> RequestActor {
        log("Spawning a new request actor");
        RequestActor { id, sender, config, root: workspace_root, cargo_handle: None, req, task_id: Default::default() }
    }

    fn report_progress(&self, progress: TaskNotification) {
        // create a notification with task_id from struct
        todo!()
    }

    fn next_event(&self, inbox: &Receiver<Event>) -> Option<Event> {
        let check_chan = self.cargo_handle.as_ref().map(|cargo| &cargo.receiver);
        select! {
            recv(inbox) -> msg => Some(Event::Cancel),
            recv(check_chan.unwrap_or(&never())) -> msg => Some(Event::CargoEvent(Some(CargoMessage{}))),
        }
    }

    pub fn run(mut self, inbox: Receiver<Event>) {
        let command = self.create_command();
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => {
                self.cargo_handle = Some(cargo_handle);
                self.report_progress(TaskNotification::Start(TaskStartParams {
                    task_id: Default::default(),
                    event_time: None,
                    message: None,
                    data: None,
                }));
            }
            Err(error) => {
                todo!()
            }
        }
        'event: while let Some(event) = self.next_event(&inbox) {
            match event {
                Cancel => {
                    self.cancel_process();
                    return;
                }
                Event::CargoEvent(None) => {
                    // tracing::debug!(flycheck_id = self.id, "flycheck finished");

                    // Watcher finished
                    let cargo_handle = self.cargo_handle.take().unwrap();
                    let res = cargo_handle.join();
                    if res.is_err() {
                        todo!()
                    }
                    self.report_progress(TaskNotification::Finish(TaskFinishParams {
                        task_id: Default::default(),
                        event_time: None,
                        message: None,
                        status: Default::default(),
                        data: None,
                    }));
                }
                Event::CargoEvent(Some(message)) => {
                    // handle information and create reponse/notification based on that
                    let resp = RPCMessage::Response(Response {
                        id: RequestId::from(0),
                        result: None,
                        error: None,
                    });
                    self.send(resp);
                }
            }
        }
    }

    fn cancel_process(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            cargo_handle.cancel();
            self.report_progress(TaskNotification::Finish(TaskFinishParams {
                task_id: Default::default(),
                event_time: None,
                message: None,
                status: Default::default(),
                data: None,
            }));
            // TODO
        }
    }

    fn create_command(&self) -> Command {
        match &self.config {
            CargoCommand::Compile(params) => { todo!() }
            CargoCommand::Debug(params) => { todo!() }
            CargoCommand::Run(params) => { todo!() }
            CargoCommand::Test(params) => { todo!() }
        };
    }

    fn send(&self, msg: RPCMessage) { (self.sender)(msg); }
}

struct JodChild(GroupChild);

/// A handle to a cargo process used for fly-checking.
struct CargoHandle {
    /// The handle to the actual cargo process. As we cannot cancel directly from with
    /// a read syscall dropping and therefore terminating the process is our best option.
    child: JodChild,
    thread: jod_thread::JoinHandle<io::Result<(bool, String)>>,
    receiver: Receiver<CargoMessage>,
}

impl CargoHandle {
    fn spawn(mut command: Command) -> std::io::Result<CargoHandle> {
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
                    self.sender.send(CargoMessage {});
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
