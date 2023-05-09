use std::path::Path;

use bsp_server::{Message, RequestId};
use crossbeam_channel::{unbounded, Sender};
use log::info;

use crate::bsp_types::requests::Request;
use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::Event;
use crate::cargo_communication::request_actor::RequestActor;

#[derive(Debug)]
pub struct RequestHandle {
    cancel_sender: Sender<Event>,
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
        let cmd = params.create_command(root_path.into());
        info!("Created command: {:?}", cmd);
        match CargoHandle::spawn(cmd) {
            Ok(cargo_handle) => {
                let (cancel_sender, cancel_receiver) = unbounded::<Event>();
                let actor: RequestActor<R, CargoHandle> = RequestActor::new(
                    sender_to_main,
                    req_id,
                    params,
                    root_path,
                    cargo_handle,
                    cancel_receiver,
                );
                let thread = jod_thread::Builder::new()
                    .spawn(move || actor.run())
                    .expect("failed to spawn thread");
                RequestHandle {
                    cancel_sender,
                    _thread: thread,
                }
            }
            Err(_err) => {
                todo!()
            }
        }
    }

    pub fn cancel(&self) {
        self.cancel_sender.send(Event::Cancel).unwrap();
    }
}
