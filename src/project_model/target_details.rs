use crate::project_model::build_target_mappings::parent_path;
use crate::project_model::cargo_package::{CargoPackage, Feature};
use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct TargetDetails<'a> {
    pub name: String,
    pub kind: CargoTargetKind,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: &'a HashSet<Feature>,
}

impl<'a, 'b> TargetDetails<'a>
where
    'b: 'a,
{
    pub fn new(
        package: &'b CargoPackage,
        target_data: &Rc<cargo_metadata::Target>,
    ) -> Option<Self> {
        Some(Self {
            name: target_data.name.clone(),
            kind: TargetDetails::get_kind(target_data)?,
            package_abs_path: parent_path(&package.manifest_path),
            default_features_disabled: package.default_features_disabled,
            enabled_features: &package.enabled_features,
        })
    }

    fn get_kind(target_data: &Rc<cargo_metadata::Target>) -> Option<CargoTargetKind> {
        serde_json::from_str::<CargoTargetKind>(target_data.kind.get(0).or_else(|| {
            error!("Invalid `kind vector` for target: {:?}", target_data.name);
            None
        })?)
        .ok()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum CargoTargetKind {
    #[default]
    Lib,
    Bin,
    Example,
    Test,
    Bench,
}

impl ToString for CargoTargetKind {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
