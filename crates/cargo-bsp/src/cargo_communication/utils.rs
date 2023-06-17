#[cfg(test)]
use cargo_metadata::{Target, TargetBuilder};
#[cfg(test)]
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use bsp_types::BuildTargetIdentifier;
use rand::distributions::{Alphanumeric, DistString};

#[cfg(test)]
use crate::project_model::cargo_package::CargoPackage;
use bsp_types::notifications::TaskId;

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

#[cfg(test)]
pub(super) fn test_target_id(uri: &str) -> BuildTargetIdentifier {
    BuildTargetIdentifier { uri: uri.into() }
}

#[cfg(test)]
pub(super) fn test_target(name: &str, kind: &str) -> Rc<Target> {
    Rc::new(
        TargetBuilder::default()
            .name(name.to_string())
            .kind(vec![kind.to_string()])
            .src_path("".to_string())
            .build()
            .unwrap(),
    )
}

#[cfg(test)]
pub(super) fn test_package(name: &str) -> CargoPackage {
    CargoPackage {
        name: name.into(),
        ..CargoPackage::default()
    }
}
