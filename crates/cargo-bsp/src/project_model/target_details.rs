use cargo_metadata::camino::Utf8PathBuf;
use std::collections::BTreeSet;

use crate::project_model::build_target_mappings::parent_path;
use log::error;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

use crate::project_model::cargo_package::{CargoPackage, Feature};

#[derive(Debug, Clone, Default)]
pub struct TargetDetails {
    pub name: String,
    pub kind: CargoTargetKind,
    pub package_abs_path: Utf8PathBuf,
    pub package_name: String,
    pub default_features_disabled: bool,
    pub enabled_features: BTreeSet<Feature>,
}

const FEATURE_FLAG: &str = "--feature ";

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
                error!("Invalid kind vector for target: {:?}", target_data.name);
                None
            })?
            .parse()
            .ok()
    }

    pub fn get_enabled_features_str(&self) -> Option<String> {
        if self.enabled_features.is_empty() {
            return None;
        }
        let enabled_features = self
            .enabled_features
            .iter()
            .map(|f| f.0.clone())
            .collect::<Vec<String>>()
            .join(", ");
        Some(FEATURE_FLAG.to_string() + enabled_features.as_str())
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
    use test_case::test_case;

    const TEST_FEATURES: [&str; 3] = ["test_feature1", "test_feature2", "test_feature3"];

    #[test_case(BTreeSet::new(), ""  ;"empty")]
    #[test_case(TEST_FEATURES.iter().map(|f| Feature(f.to_string())).collect(),
    "--feature test_feature1, test_feature2, test_feature3" ;
    "non_empty"
    )]
    fn test_get_enabled_features_string(enabled_features: BTreeSet<Feature>, expected: &str) {
        let target_details = TargetDetails {
            default_features_disabled: false,
            enabled_features,
            ..TargetDetails::default()
        };

        let enabled_features_string = target_details
            .get_enabled_features_str()
            .unwrap_or("".to_string());
        assert_eq!(enabled_features_string, expected);
    }
}
