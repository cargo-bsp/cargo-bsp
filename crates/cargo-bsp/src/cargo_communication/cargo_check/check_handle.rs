//! Creates and spawns `cargo check` command and runs a new [`CheckActor`] in
//! a new thread. Implementation of [`RequestHandle`].

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
        let root_path = global_state.config.root_path();
        let build_targets = params.get_targets(global_state.workspace);

        // The command does not need information about targets, as it is invoked with
        // `--all-targets` flag.
        let mut command = params.create_requested_command(root_path, &[]);
        let cargo_handle = CargoHandle::spawn(&mut command)?;

        let metadata = get_metadata(&global_state.config.workspace_manifest)
            .map_err(|e| io::Error::new(ErrorKind::Other, e.to_string()))?;
        let result =
            resolve_rust_workspace_result(global_state.workspace, &build_targets, &metadata);

        let (cancel_sender, cancel_receiver) = unbounded::<Event>();
        let mut actor: CheckActor<CargoHandle> =
            CheckActor::new(sender_to_main, cargo_handle, req_id, cancel_receiver);

        let thread =
            jod_thread::Builder::new().spawn(move || actor.run(result, metadata.packages))?;
        Ok(RequestHandle {
            cancel_sender,
            _thread: thread,
        })
    }
}
