//! Additional functions used within [`Execution`].

use log::warn;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use bsp4rs::bsp::{BuildTargetIdentifier, Identifier};
use rand::distributions::{Alphanumeric, DistString};

use crate::project_model::target_details::TargetDetails;
use crate::server::global_state::GlobalStateSnapshot;
use bsp4rs::bsp::TaskId;

pub(super) fn generate_random_id() -> Identifier {
    Identifier::new(Alphanumeric.sample_string(&mut rand::thread_rng(), 36))
}

pub(super) fn generate_task_id(parent: &TaskId) -> TaskId {
    TaskId {
        id: generate_random_id(),
        parents: Some(vec![parent.id.clone()]),
    }
}

pub(super) fn get_current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

pub(super) fn targets_ids_to_targets_details(
    targets_ids: &[BuildTargetIdentifier],
    global_state: &GlobalStateSnapshot,
) -> io::Result<Vec<TargetDetails>> {
    let targets_details: Vec<TargetDetails> = targets_ids
        .iter()
        .map(|id| {
            global_state
                .workspace
                .get_target_details(id)
                .ok_or_else(|| {
                    warn!("Target {:?} not found", id);
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("Target {:?} not found", id),
                    )
                })
        })
        .collect::<io::Result<Vec<TargetDetails>>>()?;
    Ok(targets_details)
}
