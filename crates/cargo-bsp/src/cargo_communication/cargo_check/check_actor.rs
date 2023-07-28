use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bsp_server::RequestId;
use bsp_server::{ErrorCode, Message, Response, ResponseError};
pub use cargo_metadata::diagnostic::{
    Applicability, Diagnostic, DiagnosticCode, DiagnosticLevel, DiagnosticSpan,
    DiagnosticSpanMacroExpansion,
};
use cargo_metadata::{BuildScript, Message as CargoMetadataMessage, PackageId};
use crossbeam_channel::{never, select, Receiver};
use log::warn;
use serde::Deserialize;
use serde_json::to_value;

use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::request_actor::CargoHandler;
use bsp_types::extensions::RustWorkspaceResult;

pub(crate) struct CheckActor<C>
where
    C: CargoHandler<CargoMessage>,
{
    // sender for notifications and responses to main loop
    pub(super) sender: Box<dyn Fn(Message) + Send>,
    pub(super) cargo_handle: Option<C>,
    cancel_receiver: Receiver<Event>,
    pub(super) req_id: RequestId,
    // TODO check if it is needed
    pub(super) _root_path: PathBuf,
}

impl<C> CheckActor<C>
where
    C: CargoHandler<CargoMessage>,
{
    pub fn new(
        sender: Box<dyn Fn(Message) + Send>,
        req_id: RequestId,
        root_path: &Path,
        cargo_handle: C,
        cancel_receiver: Receiver<Event>,
    ) -> CheckActor<C> {
        CheckActor {
            sender,
            cargo_handle: Some(cargo_handle),
            cancel_receiver,
            req_id,
            _root_path: root_path.to_path_buf(),
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

    pub fn run(&mut self, result: RustWorkspaceResult) {
        let mut build_scripts: HashMap<PackageId, BuildScript> = HashMap::new();

        while let Some(event) = self.next_event() {
            match event {
                Event::Cancel => {
                    self.cancel();
                    break;
                }
                Event::CargoFinish => {
                    self.finish(build_scripts, result);
                    break;
                }
                Event::CargoEvent(message) => {
                    self.handle_message(message, &mut build_scripts);
                }
            }
        }
    }

    fn handle_message(
        &mut self,
        message: CargoMessage,
        build_scripts: &mut HashMap<PackageId, BuildScript>,
    ) {
        match message {
            CargoMessage::CargoStdout(msg) => {
                let mut deserializer = serde_json::Deserializer::from_str(&msg);
                let message = CargoMetadataMessage::deserialize(&mut deserializer)
                    .unwrap_or(CargoMetadataMessage::TextLine(msg));
                if let CargoMetadataMessage::BuildScriptExecuted(msg) = message {
                    build_scripts.insert(msg.package_id.clone(), msg);
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
        mut result: RustWorkspaceResult,
    ) {
        let packages = result
            .packages
            .into_iter()
            .map(|p| {
                let _script = build_scripts.get(&PackageId { repr: p.id.clone() });
                // TODO get cfgOptions, env, out_dir_url, proc_macro from script (CargoMetadata 429)
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
