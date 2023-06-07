use serde::Serialize;

#[derive(Debug)]
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
