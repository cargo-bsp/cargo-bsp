//! The first handler of the compile/run/test requests after the main loop.
//! Creates and spawns all necessary commands and runs a new Check/ExecutionActor in
//! a new thread.

use crate::cargo_communication::cargo_types::event::Event;
use crossbeam_channel::Sender;

pub(crate) struct RequestHandle {
    pub(super) cancel_sender: Sender<Event>,
    pub(super) _thread: jod_thread::JoinHandle,
}
