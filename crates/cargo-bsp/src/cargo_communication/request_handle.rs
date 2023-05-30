use std::io;
use std::path::Path;
use std::process::Command;

use bsp_server::{Message, RequestId};
use crossbeam_channel::{unbounded, Sender};

use bsp_types::requests::Request;
use bsp_types::StatusCode;

use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::Event;
use crate::cargo_communication::request_actor::RequestActor;
use crate::cargo_communication::request_actor_unit_graph::UnitGraphStatusCode;

#[derive(Debug)]
pub struct RequestHandle {
    cancel_sender: Sender<Event>,
    _thread: jod_thread::JoinHandle,
}

fn run_commands<R>(mut actor: RequestActor<R, CargoHandle>, requested_cmd: Command)
where
    R: Request + 'static,
    R::Params: CreateCommand + Send,
    R::Result: CargoResult,
{
    actor.report_root_task_start();
    let unit_graph_status_code = actor.run_unit_graph();
    // We don't run requested command, if request was cancelled during
    // unit graph command.
    if let UnitGraphStatusCode::Ok = unit_graph_status_code {
        match CargoHandle::spawn(requested_cmd) {
            Ok(cargo_handle) => {
                actor.cargo_handle = Some(cargo_handle);
                actor.run();
            }
            Err(err) => {
                actor.report_task_finish(
                    actor.state.root_task_id.clone(),
                    StatusCode::Error,
                    None,
                    None,
                );
                actor.send_response(Err(err));
            }
        }
    }
}

impl RequestHandle {
    pub fn spawn<R>(
        sender_to_main: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        root_path: &Path,
    ) -> io::Result<RequestHandle>
    where
        R: Request + 'static,
        R::Params: CreateCommand + Send,
        R::Result: CargoResult,
    {
        let unit_graph_cmd = params.create_unit_graph_command(root_path.into());
        let requested_cmd = params.create_requested_command(root_path.into());

        let cargo_handle = CargoHandle::spawn(unit_graph_cmd)?;
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();
        let actor: RequestActor<R, CargoHandle> = RequestActor::new(
            sender_to_main,
            req_id,
            params,
            root_path,
            cargo_handle,
            cancel_receiver,
        );
        let thread =
            jod_thread::Builder::new().spawn(move || run_commands(actor, requested_cmd))?;
        Ok(RequestHandle {
            cancel_sender,
            _thread: thread,
        })
    }

    pub fn cancel(&self) {
        self.cancel_sender.send(Event::Cancel).unwrap();
    }
}
