//! Additional functions used within [`CargoCommunication`] and during its testing.

#[cfg(test)]
use bsp_types::BuildTargetIdentifier;
#[cfg(test)]
use cargo_metadata::{Target, TargetBuilder};
#[cfg(test)]
use std::rc::Rc;

#[cfg(test)]
use crate::project_model::cargo_package::CargoPackage;

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
