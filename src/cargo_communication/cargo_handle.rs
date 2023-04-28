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

use crate::cargo_communication::cargo_actor::CargoActor;
use crate::cargo_communication::cargo_types::event::CargoMessage;
use crate::cargo_communication::request_actor::CargoHandleTrait;

pub struct CargoHandle {
    /// The handle to the actual cargo process. As we cannot cancel directly from with
    /// a read syscall dropping and therefore terminating the process is our best option.
    child: GroupChild,
    thread: jod_thread::JoinHandle<io::Result<bool>>,
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
    pub fn spawn(mut command: Command) -> io::Result<CargoHandle> {
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
