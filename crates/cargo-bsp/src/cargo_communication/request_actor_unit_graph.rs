use bsp_types::requests::Request;
use bsp_types::StatusCode;
use log::warn;
use serde::Deserialize;

use crate::cargo_communication::cargo_types::cargo_command::CreateCommand;
use crate::cargo_communication::cargo_types::cargo_result::CargoResult;
use crate::cargo_communication::cargo_types::event::{CargoMessage, Event};
use crate::cargo_communication::cargo_types::unit_graph::UnitGraph;
use crate::cargo_communication::request_actor::{CargoHandler, RequestActor};

pub enum UnitGraphStatusCode {
    Ok,
    Cancelled,
}

impl<R, C> RequestActor<R, C>
where
    R: Request,
    R::Params: CreateCommand,
    R::Result: CargoResult,
    C: CargoHandler<CargoMessage>,
{
    pub fn run_unit_graph(&mut self) -> UnitGraphStatusCode {
        self.report_task_start(self.state.root_task_id.clone(), None, None);
        self.report_task_start(
            self.state.unit_graph_state.task_id.clone(),
            Some("Started unit graph command".to_string()),
            None,
        );

        let mut received_unit_graph = false;

        while let Some(event) = self.next_event() {
            match event {
                Event::Cancel => {
                    self.cancel();
                    return UnitGraphStatusCode::Cancelled;
                }
                Event::CargoFinish => {
                    self.finish_unit_graph(received_unit_graph);
                    break;
                }
                Event::CargoEvent(message) => {
                    self.handle_unit_graph_message(message, &mut received_unit_graph)
                }
            }
        }
        UnitGraphStatusCode::Ok
    }

    pub(super) fn handle_unit_graph_message(
        &mut self,
        message: CargoMessage,
        received_unit_graph: &mut bool,
    ) {
        match message {
            CargoMessage::CargoStdout(msg) => {
                if *received_unit_graph {
                    warn!(
                        "Received other message than unit graph from unit graph command: {}",
                        msg
                    );
                } else {
                    self.deserialize_message_from_unit_graph(msg, received_unit_graph);
                }
            }
            CargoMessage::CargoStderr(msg) => {
                warn!("Error message from unit graph command: {}", msg);
            }
        }
    }

    fn deserialize_message_from_unit_graph(&mut self, msg: String, received_unit_graph: &mut bool) {
        let mut deserializer = serde_json::Deserializer::from_str(&msg);
        match UnitGraph::deserialize(&mut deserializer) {
            Ok(unit_graph) => {
                *received_unit_graph = true;
                self.state.unit_graph_state.total_compilation_steps =
                    Some(unit_graph.get_compilation_steps());
                self.state.compile_state.compilation_step = Some(0);
            }
            Err(e) => {
                warn!(
                    "Could not parse a message from cargo unit graph command: {}",
                    e.to_string()
                );
            }
        };
    }

    fn finish_unit_graph(&mut self, received_unit_graph: bool) {
        let _ = self.cargo_handle.take().unwrap().join();

        let status_code = if !received_unit_graph {
            warn!("Didn't receive unit graph from unit graph command");
            StatusCode::Error
        } else {
            StatusCode::Ok
        };
        self.report_task_finish(
            self.state.unit_graph_state.task_id.clone(),
            status_code,
            Some("Finished unit graph command".to_string()),
            None,
        );
    }
}
