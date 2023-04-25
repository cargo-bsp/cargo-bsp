use crate::bsp_types::requests::Request;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::Event;
use crate::cargo_communication::request_actor::RequestActor;
use crate::communication::{Message, RequestId};
use crossbeam_channel::{unbounded, Sender};
use std::path::Path;

#[derive(Debug)]
pub struct RequestHandle {
    sender_to_cancel: Sender<Event>,
    _thread: jod_thread::JoinHandle,
}

impl RequestHandle {
    pub fn spawn<R>(
        sender_to_main: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> RequestHandle
    where
        R: Request + 'static,
        R::Params: CreateCommand + Send,
        R::Result: CargoResult,
    {
        let mut actor: RequestActor<R> =
            RequestActor::new(sender_to_main, req_id, params, root_path);
        let (sender_to_cancel, receiver_to_cancel) = unbounded::<Event>();
        let thread = jod_thread::Builder::new()
            .spawn(move || match actor.spawn_cargo_handle() {
                Ok(cargo_handle) => actor.run(receiver_to_cancel, cargo_handle),
                Err(_err) => {
                    todo!()
                }
            })
            .expect("failed to spawn thread");
        RequestHandle {
            sender_to_cancel,
            _thread: thread,
        }
    }

    pub fn cancel(&self) {
        self.sender_to_cancel.send(Event::Cancel).unwrap();
    }
}
