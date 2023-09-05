//! Creates and spawns `cargo check` command and runs a new [`ExecutionActor`] in
//! a new thread. Implementation of [`RequestHandle`].

use std::io;
use std::process::Command;

use bsp_server::{Message, RequestId};
use crossbeam_channel::unbounded;

use bsp_types::requests::Request;
use bsp_types::StatusCode;

use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::event::Event;
use crate::cargo_communication::cargo_types::params_target::ParamsTarget;
use crate::cargo_communication::execution::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::execution::cargo_types::cargo_unit_graph_command::CreateUnitGraphCommand;
use crate::cargo_communication::execution::cargo_types::origin_id::OriginId;
use crate::cargo_communication::execution::execution_actor::ExecutionActor;
use crate::cargo_communication::execution::execution_actor_unit_graph::UnitGraphStatusCode;
use crate::cargo_communication::execution::utils::targets_ids_to_targets_details;
use crate::cargo_communication::request_handle::RequestHandle;
use crate::server::global_state::GlobalStateSnapshot;

impl RequestHandle {
    pub fn spawn<R>(
        sender_to_main: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        global_state: GlobalStateSnapshot,
    ) -> io::Result<RequestHandle>
    where
        R: Request + 'static,
        R::Params: CreateUnitGraphCommand + CreateCommand + ParamsTarget + OriginId + Send,
        R::Result: CargoResult,
    {
        let root_path = global_state.config.root_path();
        let targets_details = targets_ids_to_targets_details(
            &params.get_targets(global_state.workspace),
            &global_state,
        )?;
        let mut unit_graph_cmd = params.create_unit_graph_command(root_path, &targets_details);
        let mut requested_cmd = params.create_requested_command(root_path, &targets_details);
        let cargo_handle = CargoHandle::spawn(&mut unit_graph_cmd)?;
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();
        let actor: ExecutionActor<R, CargoHandle> = ExecutionActor::new(
            sender_to_main,
            req_id,
            params,
            root_path,
            cargo_handle,
            cancel_receiver,
            global_state.workspace,
        );
        let thread =
            jod_thread::Builder::new().spawn(move || run_commands(actor, &mut requested_cmd))?;
        Ok(RequestHandle {
            cancel_sender,
            _thread: thread,
        })
    }

    pub fn cancel(&self) {
        self.cancel_sender.send(Event::Cancel).unwrap();
    }
}

fn run_commands<R>(mut actor: ExecutionActor<R, CargoHandle>, requested_cmd: &mut Command)
where
    R: Request + 'static,
    R::Params: CreateUnitGraphCommand + ParamsTarget + OriginId + Send,
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
