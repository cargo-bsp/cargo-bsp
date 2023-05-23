use std::collections::BTreeSet;

use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::project_model::build_target_mappings::parent_path;
use crate::project_model::cargo_package::{CargoPackage, Feature};

#[derive(Debug, Clone)]
pub struct TargetDetails {
    pub name: String,
    pub kind: CargoTargetKind,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: BTreeSet<Feature>,
}

impl Default for TargetDetails {
    fn default() -> Self {
        Self {
            name: String::new(),
            kind: CargoTargetKind::Lib,
            package_name: String::new(),
            default_features_disabled: false,
            enabled_features: BTreeSet::new(),
        }
    }
}

impl<'a> TargetDetails {
    pub fn new(package: &'a CargoPackage, target_data: &cargo_metadata::Target) -> Option<Self> {
        Some(Self {
            name: target_data.name.clone(),
            kind: TargetDetails::get_kind(target_data)?,
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
                error!("Invalid `kind vector` for target: {:?}", target_data.name);
                None
            })?
            .parse()
            .ok()
    }

    pub fn get_enabled_features_str(&self) -> String {
        if self.enabled_features.is_empty() {
            return String::new();
        }
        let enabled_features = self
            .enabled_features
            .iter()
            .map(|f| f.0.clone())
            .collect::<Vec<String>>()
            .join(", ");
        "--feature ".to_string() + enabled_features.as_str()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_enabled_features_string_empty() {
        let target_details = TargetDetails {
            name: "test_target".to_string(),
            kind: CargoTargetKind::Test,
            package_name: "test_package".to_string(),
            default_features_disabled: false,
            enabled_features: BTreeSet::new(),
        };

        let enabled_features_string = target_details.get_enabled_features_str();
        assert_eq!(enabled_features_string, "");
    }

    #[test]
    fn test_get_enabled_features_string_non_empty() {
        let target_details = TargetDetails {
            name: "test_target".to_string(),
            kind: CargoTargetKind::Test,
            package_name: "test_package".to_string(),
            default_features_disabled: false,
            enabled_features: vec![
                Feature("test_feature1".to_string()),
                Feature("test_feature2".to_string()),
                Feature("test_feature3".to_string()),
            ]
            .into_iter()
            .collect(),
        };

        let enabled_features_str = target_details.get_enabled_features_str();
        assert_eq!(
            enabled_features_str,
            "--feature test_feature1, test_feature2, test_feature3"
        );
    }
}
