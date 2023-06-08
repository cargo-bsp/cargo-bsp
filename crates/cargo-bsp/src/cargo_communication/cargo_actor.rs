use std::{
    io,
    process::{ChildStderr, ChildStdout},
};

pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use crossbeam_channel::Sender;
use log::warn;
use stdx::process::streaming_output;

use crate::cargo_communication::cargo_types::event::CargoMessage;

pub struct CargoActor {
    sender: Sender<CargoMessage>,
    stdout: ChildStdout,
    stderr: ChildStderr,
}

impl CargoActor {
    pub fn new(
        sender: Sender<CargoMessage>,
        stdout: ChildStdout,
        stderr: ChildStderr,
    ) -> CargoActor {
        CargoActor {
            sender,
            stdout,
            stderr,
        }
    }

    pub fn run(self) -> io::Result<bool> {
        // We manually read a line at a time, instead of using serde's
        // stream deserializers, because the deserializer cannot recover
        // from an error, resulting in it getting stuck, because we try to
        // be resilient against failures.
        //
        // Because cargo only outputs one JSON object per line, we can
        // simply skip a line if it doesn't parse, which just ignores any
        // erroneous output.
        //
        // We return bool that indicates whether we read at least one message.

        let mut read_at_least_one_message = false;
        let output = streaming_output(
            self.stdout,
            self.stderr,
            &mut |line| {
                read_at_least_one_message = true;
                self.sender
                    .send(CargoMessage::CargoStdout(line.to_string()))
                    .unwrap_or_else(|e| {
                        warn!("Could not send a message from cargo: {}", e.to_string());
                    });
            },
            &mut |line| {
                self.sender
                    .send(CargoMessage::CargoStderr(line.to_string()))
                    .unwrap_or_else(|e| {
                        warn!("Could not send a message from cargo: {}", e.to_string());
                    });
            },
        );
        match output {
            Ok(_) => Ok(read_at_least_one_message),
            Err(e) => Err(io::Error::new(e.kind(), format!("{:?}", e))),
        }
    }
}
