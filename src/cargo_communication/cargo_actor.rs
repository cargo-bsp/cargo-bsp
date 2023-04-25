use std::{
    io,
    process::{ChildStderr, ChildStdout},
};

use crossbeam_channel::Sender;
use log::warn;
use serde::Deserialize;
use stdx::process::streaming_output;

use crate::cargo_communication::cargo_types::event::CargoMessage;
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use cargo_metadata::Message;

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
                        warn!("Could not parse a message from cargo: {}", e.to_string());
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
