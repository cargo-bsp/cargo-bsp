use crate::bsp_types::cargo_output::Message;
use std::process::ExitStatus;
use std::{
    io,
    process::{ChildStderr, ChildStdout, Command, Stdio},
};
// pub use cargo_metadata::diagnostic::{
//     Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
//     DiagnosticSpanMacroExpansion,
// };
// use cargo_metadata::Message;
use crate::server::request_actor::{CargoHandleTrait, CargoMessage};
use command_group::{CommandGroup, GroupChild};
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde::Deserialize;
use stdx::process::streaming_output;

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

    fn run(self) -> io::Result<bool> {
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

        let mut read_at_least_one_message = false;
        let output = streaming_output(
            self.stdout,
            self.stderr,
            &mut |line| {
                read_at_least_one_message = true;

                // Try to deserialize a message from Cargo.
                let mut deserializer = serde_json::Deserializer::from_str(line);
                deserializer.disable_recursion_limit();
                match Message::deserialize(&mut deserializer) {
                    Ok(message) => {
                        self.sender
                            .send(CargoMessage::CargoStdout(message))
                            .expect("TODO: panic message");
                    }
                    Err(e) => {
                        // todo!("Log that we couldn't parse a message: {:?}", line")
                    }
                };
            },
            &mut |line| {
                self.sender
                    .send(CargoMessage::CargoStderr(line.to_string()))
                    .expect("TODO: panic message");
            },
        );
        match output {
            Ok(_) => Ok(read_at_least_one_message),
            Err(e) => Err(io::Error::new(e.kind(), format!("{:?}", e))),
        }
    }
}
