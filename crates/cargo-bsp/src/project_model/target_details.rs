use std::collections::BTreeSet;

use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::project_model::build_target_mappings::parent_path;
use crate::project_model::cargo_package::{CargoPackage, Feature};

#[derive(Debug, Clone)]
pub struct TargetDetails<'a> {
    pub name: String,
    pub kind: CargoTargetKind,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: &'a BTreeSet<Feature>,
}

impl<'a> TargetDetails<'a> {
    pub fn new(package: &'a CargoPackage, target_data: &cargo_metadata::Target) -> Option<Self> {
        Some(Self {
            name: target_data.name.clone(),
            kind: TargetDetails::get_kind(target_data)?,
            package_abs_path: parent_path(&package.manifest_path),
            default_features_disabled: package.default_features_disabled,
            enabled_features: &package.enabled_features,
        })
    }

    fn get_kind(target_data: &cargo_metadata::Target) -> Option<CargoTargetKind> {
        target_data
            .kind
            .get(0)
            .or_else(|| {
                error!("Invalid `kind vector` for target: {:?}", target_data.name);
                None
            })?
            .parse()
            .ok()
    }
}

#[derive(Debug, Deserialize_enum_str, Serialize_enum_str, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub enum CargoTargetKind {
    #[default]
    Lib,
    Bin,
    Example,
    Test,
    Bench,
}
