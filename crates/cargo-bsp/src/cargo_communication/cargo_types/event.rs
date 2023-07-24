use serde::Serialize;

/// Indicates, where the message is coming from:
/// - Cancel when the client canceled the request,
/// - CargoEvent when there is a new message from Cargo (from stdout or stderr),
/// - CargoFinish when Cargo command finished its execution.
pub enum Event {
    Cancel,
    CargoEvent(CargoMessage),
    CargoFinish,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum CargoMessage {
    CargoStdout(String),
    CargoStderr(String),
}
