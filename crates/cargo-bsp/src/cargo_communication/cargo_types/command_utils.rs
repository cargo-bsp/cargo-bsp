//! Contains necessary additional structs and functions for creating Cargo commands.

use crate::project_model::target_details::TargetDetails;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use std::ops::Deref;

#[derive(Debug, Deserialize_enum_str, Serialize_enum_str, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CommandType {
    Build,
    Test,
    Run,
    Check,
}

const FEATURE_FLAG: &str = "--features";

impl TargetDetails {
    pub fn get_enabled_features_str(&self) -> Option<String> {
        let only_default_feature_enabled =
            self.enabled_features.len() == 1 && !self.default_features_disabled();
        match self.enabled_features.is_empty() || only_default_feature_enabled {
            true => None,
            false => Some(
                self.enabled_features
                    .iter()
                    .filter_map(|f| match f.as_str() {
                        "default" => None,
                        _ => Some(f.deref().clone()),
                    })
                    .collect::<Vec<String>>()
                    .join(", "),
            ),
        }
    }
}

/// Creates additional flags for the command to specify the packages, targets and features.
pub(crate) fn targets_details_to_args(targets_details: &[TargetDetails]) -> Vec<String> {
    targets_details
        .iter()
        .flat_map(|t| {
            let mut loc_args = Vec::new();
            loc_args.push("--package".to_string());
            loc_args.push(t.package_name.clone());
            if t.kind.is_lib() {
                loc_args.push("--lib".to_string());
            } else {
                loc_args.push(format!("--{}", t.kind));
                loc_args.push(t.name.clone());
            }
            if let Some(features) = t.get_enabled_features_str() {
                loc_args.push(FEATURE_FLAG.to_string());
                loc_args.push(features);
            }
            if t.default_features_disabled() {
                loc_args.push("--no-default-features".to_string());
            }
            loc_args
        })
        .collect()
}
