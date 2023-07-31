use std::collections::hash_map::Entry;
use std::collections::HashMap;

use bsp_server::RequestId;
use bsp_server::{ErrorCode, Message, Response, ResponseError};
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use cargo_metadata::{Artifact, BuildScript, Message as CargoMetadataMessage, Package, PackageId};
use crossbeam_channel::{never, select, Receiver};
use log::warn;
use serde::Deserialize;
use serde_json::to_value;

use crate::cargo_communication::cargo_check::build_script_to_package_info::{
    map_cfg_options, map_env, map_out_dir_url, map_proc_macro_artifact,
};
use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::request_actor::CargoHandler;
use bsp_types::extensions::RustWorkspaceResult;

pub(crate) struct CheckActor<C>
where
    C: CargoHandler<CargoMessage>,
{
    // sender for notifications and responses to main loop
    sender: Box<dyn Fn(Message) + Send>,
    cargo_handle: Option<C>,
    cancel_receiver: Receiver<Event>,
    req_id: RequestId,
}

impl<C> CheckActor<C>
where
    C: CargoHandler<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        cargo_handle: C,
        cancel_receiver: Receiver<Event>,
    ) -> CheckActor<C> {
        CheckActor {
            sender,
            cargo_handle: Some(cargo_handle),
            cancel_receiver,
            req_id,
        }
    }

    fn next_event(&self) -> Option<Event> {
        let cargo_chan = self.cargo_handle.as_ref().map(|cargo| cargo.receiver());
        select! {
            recv(self.cancel_receiver) -> msg => msg.ok(),
            recv(cargo_chan.unwrap_or(&never())) -> msg => match msg {
                Ok(msg) => Some(Event::CargoEvent(msg)),
                Err(_) => Some(Event::CargoFinish),
            }
        }
    }

    pub fn run(&mut self, result: RustWorkspaceResult, packages: Vec<Package>) {
        let mut build_scripts: HashMap<PackageId, BuildScript> = HashMap::new();
        let mut compiler_artifacts: HashMap<PackageId, Vec<Artifact>> = HashMap::new();

        while let Some(event) = self.next_event() {
            match event {
                Event::Cancel => {
                    self.cancel();
                    break;
                }
                Event::CargoFinish => {
                    self.finish(build_scripts, compiler_artifacts, result, packages);
                    break;
                }
                Event::CargoEvent(message) => {
                    self.handle_message(message, &mut build_scripts, &mut compiler_artifacts);
                }
            }
        }
    }

    fn handle_message(
        &mut self,
        message: CargoMessage,
        build_scripts: &mut HashMap<PackageId, BuildScript>,
        compiler_artifacts: &mut HashMap<PackageId, Vec<Artifact>>,
    ) {
        match message {
            CargoMessage::CargoStdout(msg) => {
                let mut deserializer = serde_json::Deserializer::from_str(&msg);
                let message = CargoMetadataMessage::deserialize(&mut deserializer)
                    .unwrap_or(CargoMetadataMessage::TextLine(msg));
                match message {
                    CargoMetadataMessage::BuildScriptExecuted(msg) => {
                        build_scripts.insert(msg.package_id.clone(), msg);
                    }
                    CargoMetadataMessage::CompilerArtifact(msg) => {
                        if let Entry::Vacant(e) = compiler_artifacts.entry(msg.package_id.clone()) {
                            e.insert(vec![msg]);
                        } else {
                            compiler_artifacts
                                .get_mut(&msg.package_id)
                                .unwrap()
                                .push(msg);
                        }
                    }
                    _ => {}
                }
            }
            CargoMessage::CargoStderr(msg) => {
                warn!("Error message from check command: {}", msg);
            }
        }
    }

    fn finish(
        &mut self,
        build_scripts: HashMap<PackageId, BuildScript>,
        compiler_artifacts: HashMap<PackageId, Vec<Artifact>>,
        mut result: RustWorkspaceResult,
        packages: Vec<Package>,
    ) {
        let packages = result
            .packages
            .into_iter()
            .map(|mut p| {
                let package_id = PackageId { repr: p.id.clone() };
                let script = build_scripts.get(&package_id);
                let default_artifacts = &vec![];
                let artifacts = compiler_artifacts
                    .get(&package_id)
                    .unwrap_or(default_artifacts);
                // We can unwrap, as there would be no RustWorkspaceResult without this package.
                let package = packages.iter().find(|&p| p.id == package_id).unwrap();

                // TODO get cfgOptions, env, out_dir_url, proc_macro from script (CargoMetadata 429)
                p.cfg_options = map_cfg_options(script);
                p.env = map_env(script, package);
                p.out_dir_url = map_out_dir_url(script);
                p.proc_macro_artifact = map_proc_macro_artifact(artifacts);
                p
            })
            .collect();

        result.packages = packages;
        self.send(
            Response {
                id: self.req_id.clone(),
                result: Some(to_value(result).unwrap()),
                error: None,
            }
            .into(),
        );
    }

    fn cancel(&mut self) {
        if let Some(cargo_handle) = self.cargo_handle.take() {
            cargo_handle.cancel();
            let error = ResponseError {
                code: ErrorCode::RequestCanceled as i32,
                message: "canceled by client".to_string(),
                data: None,
            };
            self.send(
                Response {
                    id: self.req_id.clone(),
                    result: None,
                    error: Some(error),
                }
                .into(),
            );
        } else {
            warn!(
                "Tried to cancel request {} that was already finished",
                self.req_id.clone()
            );
        }
    }

    fn send(&self, msg: Message) {
        (self.sender)(msg);
    }
}