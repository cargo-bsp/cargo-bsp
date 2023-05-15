use cargo_metadata::Message;
use serde::Serialize;

pub enum Event {
    Cancel,
    CargoEvent(CargoMessage),
    CargoFinish,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum CargoMessage {
    CargoStdout(Message),
    CargoStderr(String),
}
