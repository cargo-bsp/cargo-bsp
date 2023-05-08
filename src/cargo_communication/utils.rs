use crate::bsp_types::notifications::TaskId;
use rand::distributions::{Alphanumeric, DistString};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn generate_random_id() -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), 36)
}

pub(super) fn generate_task_id(parent: &TaskId) -> TaskId {
    TaskId {
        id: generate_random_id(),
        parents: vec![parent.id.clone()],
    }
}

pub(super) fn get_current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}
