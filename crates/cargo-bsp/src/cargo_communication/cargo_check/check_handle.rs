use std::io;
use std::io::ErrorKind;
use std::process::Command;

use bsp_server::{Message, RequestId};
use bsp_types::extensions::RustWorkspaceResult;
use cargo_metadata::Package;
use crossbeam_channel::unbounded;

use crate::cargo_communication::cargo_check::check_actor::CheckActor;
use bsp_types::requests::Request;

use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::event::Event;
use crate::cargo_communication::cargo_types::params_target::ParamsTarget;
use crate::cargo_communication::request_handle::RequestHandle;
use crate::cargo_communication::utils::targets_ids_to_targets_details;
use crate::project_model::rust_extension::{get_metadata, resolve_rust_workspace_result};
use crate::server::global_state::GlobalStateSnapshot;

impl RequestHandle {
    pub fn spawn_check<R>(
        sender_to_main: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        params: R::Params,
        global_state: GlobalStateSnapshot,
    ) -> io::Result<RequestHandle>
    where
        R: Request + 'static,
        R::Params: CreateCommand + ParamsTarget + Send,
    {
        // TODO section off similar parts with spawn method
        let root_path = global_state.config.root_path();
        let build_targets = params.get_targets(&global_state.workspace);
        let targets_details = targets_ids_to_targets_details(&build_targets, &global_state)?;
        let commands = params.create_requested_command(root_path, &targets_details);
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();
        let metadata = get_metadata(&global_state.config.workspace_manifest)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
        let actor: CheckActor<CargoHandle> =
            CheckActor::new(sender_to_main, req_id, cancel_receiver);
        let result =
            resolve_rust_workspace_result(&global_state.workspace, &build_targets, &metadata);
        let thread = jod_thread::Builder::new()
            .spawn(move || run_check_commands(actor, commands, result, metadata.packages))?;
        Ok(RequestHandle {
            cancel_sender,
            _thread: thread,
        })
    }
}

fn run_check_commands(
    mut actor: CheckActor<CargoHandle>,
    mut commands: Vec<Command>,
    result: RustWorkspaceResult,
    packages: Vec<Package>,
) {
    for command in &mut commands {
        match CargoHandle::spawn(command) {
            Ok(cargo_handle) => {
                actor.cargo_handle = Some(cargo_handle);
                actor.run();
            }
            Err(err) => {
                actor.send_response(Err(err));
            }
        }
    }
    actor.finish(result, packages);
}
