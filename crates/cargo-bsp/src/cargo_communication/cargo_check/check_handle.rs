use std::io;
use std::io::ErrorKind;

use bsp_server::{Message, RequestId};
use crossbeam_channel::unbounded;

use crate::cargo_communication::cargo_check::check_actor::CheckActor;
use bsp_types::requests::Request;

use crate::cargo_communication::cargo_handle::CargoHandle;
use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::event::Event;
use crate::cargo_communication::cargo_types::params_target::ParamsTarget;
use crate::cargo_communication::request_handle::RequestHandle;
use crate::cargo_communication::utils::targets_ids_to_targets_details;
use crate::project_model::rust_extension::resolve_rust_workspace_result;
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
        let targets_details = targets_ids_to_targets_details(
            params.get_targets(&global_state.workspace),
            &global_state,
        )?;
        let command = params.create_requested_command(root_path, &targets_details);
        let cargo_handle = CargoHandle::spawn(command)?;
        let (cancel_sender, cancel_receiver) = unbounded::<Event>();
        let mut actor: CheckActor<CargoHandle> = CheckActor::new(
            sender_to_main,
            req_id,
            root_path,
            cargo_handle,
            cancel_receiver,
        );
        let build_targets = params.get_targets(&global_state.workspace);
        let result = resolve_rust_workspace_result(
            &global_state.workspace,
            &global_state.config.workspace_manifest,
            &build_targets,
        )
        .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
        let thread = jod_thread::Builder::new().spawn(move || actor.run(result))?;
        Ok(RequestHandle {
            cancel_sender,
            _thread: thread,
        })
    }
}
