//! Spawns and handles Cargo command.
//! Wraps around the communication needed to be able to run `cargo build/run/test`
//! without blocking. Currently the Rust standard library doesn't provide a way to read sub-process
//! output without blocking, so we have to wrap sub-processes output handling in a thread and pass
//! messages back over a channel (see [`CargoActor`]).

use std::process::ExitStatus;
use std::{
    io,
    process::{Command, Stdio},
};

pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use command_group::{CommandGroup, GroupChild};
use crossbeam_channel::{unbounded, Receiver};
use log::info;

use crate::cargo_communication::cargo_actor::CargoActor;
use crate::cargo_communication::cargo_types::event::CargoMessage;
use crate::cargo_communication::execution::execution_actor::CargoHandler;

pub struct CargoHandle {
    /// The handle to the actual cargo process. As we cannot cancel directly from with
    /// a read syscall dropping and therefore terminating the process is our best option.
    child: GroupChild,
    thread: jod_thread::JoinHandle<io::Result<bool>>,
    receiver: Receiver<CargoMessage>,
}

impl CargoHandler<CargoMessage> for CargoHandle {
    fn receiver(&self) -> &Receiver<CargoMessage> {
        &self.receiver
    }

    fn cancel(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn join(mut self) -> io::Result<ExitStatus> {
        let _ = self.child.kill();
        let exit_status = self.child.wait()?;
        let read_at_least_one_message = self.thread.join()?;
        if read_at_least_one_message {
            Ok(exit_status)
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "Cargo watcher failed, the command produced no valid metadata (exit code: {:?}):\n",
                    exit_status
                ),
            ))
        }
    }
}

impl CargoHandle {
    pub fn spawn(command: &mut Command) -> io::Result<CargoHandle> {
        info!("Created command: {:?}", command);
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        let mut child = command.group_spawn()?;

        let stdout = child.inner().stdout.take().unwrap();
        let stderr = child.inner().stderr.take().unwrap();

        let (sender, receiver) = unbounded();
        let actor = CargoActor::new(sender, stdout, stderr);
        let thread = jod_thread::Builder::new().spawn(move || actor.run())?;
        Ok(CargoHandle {
            child,
            thread,
            receiver,
        })
    }
}
