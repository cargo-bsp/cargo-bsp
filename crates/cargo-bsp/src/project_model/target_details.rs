use std::collections::BTreeSet;

use bsp_types::requests::Feature;
use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::project_model::build_target_mappings::parent_path;
use crate::project_model::cargo_package::CargoPackage;

#[derive(Debug, Clone, Default)]
pub struct TargetDetails {
    pub name: String,
    pub kind: CargoTargetKind,
    pub package_abs_path: Utf8PathBuf,
    pub package_name: String,
    pub default_features_disabled: bool,
    pub enabled_features: BTreeSet<Feature>,
}

impl TargetDetails {
    pub fn new(package: &CargoPackage, target_data: &cargo_metadata::Target) -> Option<Self> {
        Some(Self {
            name: target_data.name.clone(),
            kind: TargetDetails::get_kind(target_data)?,
            package_abs_path: parent_path(&package.manifest_path),
            package_name: package.name.to_string(),
            default_features_disabled: package.default_features_disabled,
            enabled_features: package.enabled_features.clone(),
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

#[derive(Debug, Deserialize_enum_str, Serialize_enum_str, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CargoTargetKind {
    #[default]
    Lib,
    Bin,
    Example,
    Test,
    Bench,
}
