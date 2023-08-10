//! Stores Cargo target details.

use std::collections::BTreeSet;

use bsp_types::extensions::Feature;
use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::project_model::build_target_mappings::parent_path;
use crate::project_model::cargo_package::CargoPackage;

/// The order resembles Cargo's target structure.
/// Specifically, package_name, kind and name are in the same order as it is in Cargo.
/// This order should not be changed and new fields should be added at the end.
#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct TargetDetails {
    pub package_name: String,
    pub kind: CargoTargetKind,
    pub name: String,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: BTreeSet<Feature>,
}

impl TargetDetails {
    pub fn new(package: &CargoPackage, target_data: &cargo_metadata::Target) -> Option<Self> {
        Some(Self {
            package_name: package.name.to_string(),
            kind: TargetDetails::get_kind(target_data)?,
            name: target_data.name.clone(),
            package_abs_path: parent_path(&package.manifest_path),
            default_features_disabled: package.default_features_disabled,
            enabled_features: package.enabled_features.clone(),
        })
    }

    fn get_kind(target_data: &cargo_metadata::Target) -> Option<CargoTargetKind> {
        target_data
            .kind
            .get(0)
            .or_else(|| {
                error!("Invalid kind vector for target: {:?}", target_data.name);
                None
            })?
            .parse()
            .ok()
    }
}

/// The order in enum is the same as it is in Cargo.
/// This order should not be changed and new fields should be added to match the structure in Cargo.
#[derive(
    Debug, Deserialize_enum_str, Serialize_enum_str, Default, Clone, Ord, PartialOrd, Eq, PartialEq,
)]
#[serde(rename_all = "camelCase")]
pub enum CargoTargetKind {
    #[default]
    Lib,
    Bin,
    Test,
    Bench,
    Example,
}
