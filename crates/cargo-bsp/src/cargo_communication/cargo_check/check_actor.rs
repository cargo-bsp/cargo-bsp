//! Handles messages from Cargo check command, parsing them and preparing appropriate
//! response. Also handles information about the finish of Cargo command and
//! the cancel request from the client.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;

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

use crate::cargo_communication::cargo_check::cargo_message_to_package_info::{
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
    build_scripts: HashMap<PackageId, BuildScript>,
    compiler_artifacts: HashMap<PackageId, Vec<Artifact>>,
}

impl<C> CheckActor<C>
where
    C: CargoHandler<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(Message) + Send>,
        cargo_handle: C,
        req_id: RequestId,
        cancel_receiver: Receiver<Event>,
    ) -> CheckActor<C> {
        CheckActor {
            sender,
            cargo_handle: Some(cargo_handle),
            cancel_receiver,
            req_id,
            build_scripts: HashMap::new(),
            compiler_artifacts: HashMap::new(),
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
        while let Some(event) = self.next_event() {
            match event {
                Event::Cancel => {
                    self.cancel();
                    return;
                }
                Event::CargoFinish => {
                    break;
                }
                Event::CargoEvent(message) => {
                    self.handle_message(message);
                }
            }
        }
        self.finish(result, packages);
    }

    fn handle_message(&mut self, message: CargoMessage) {
        match message {
            CargoMessage::CargoStdout(msg) => {
                let mut deserializer = serde_json::Deserializer::from_str(&msg);
                let message = CargoMetadataMessage::deserialize(&mut deserializer)
                    .unwrap_or(CargoMetadataMessage::TextLine(msg));
                match message {
                    CargoMetadataMessage::BuildScriptExecuted(msg) => {
                        self.build_scripts.insert(msg.package_id.clone(), msg);
                    }
                    CargoMetadataMessage::CompilerArtifact(msg) => {
                        if let Entry::Vacant(e) =
                            self.compiler_artifacts.entry(msg.package_id.clone())
                        {
                            e.insert(vec![msg]);
                        } else {
                            self.compiler_artifacts
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

    pub(super) fn finish(&mut self, mut result: RustWorkspaceResult, packages: Vec<Package>) {
        let packages = result
            .packages
            .into_iter()
            .map(|mut p| {
                let package_id = PackageId { repr: p.id.clone() };
                let script = self.build_scripts.get(&package_id);
                let default_artifacts = &vec![];
                let artifacts = self
                    .compiler_artifacts
                    .get(&package_id)
                    .unwrap_or(default_artifacts);
                // We can unwrap, as there would be no RustWorkspaceResult without this package.
                let package = packages.iter().find(|&p| p.id == package_id).unwrap();

                p.cfg_options = map_cfg_options(script);
                p.env = map_env(script, package);
                p.out_dir_url = map_out_dir_url(script);
                p.proc_macro_artifact = map_proc_macro_artifact(artifacts);
                p
            })
            .collect();

        result.packages = packages;
        self.send_response(Ok(result));
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

    pub(super) fn send_response(&self, command_result: io::Result<RustWorkspaceResult>) {
        self.send(
            Response {
                id: self.req_id.clone(),
                result: command_result
                    .as_ref()
                    .ok()
                    .map(|result| to_value(result).unwrap()),
                error: command_result.as_ref().err().map(|e| ResponseError {
                    code: ErrorCode::InternalError as i32,
                    message: e.to_string(),
                    data: None,
                }),
            }
            .into(),
        );
    }

    fn send(&self, msg: Message) {
        (self.sender)(msg);
    }
}
