use crate::requests::cargo_extension::Feature;
use serde::{Deserialize, Serialize};

use crate::requests::Request;

#[derive(Debug)]
pub enum DisableRustFeatures {}

impl Request for DisableRustFeatures {
    type Params = DisableRustFeaturesParams;
    type Result = DisableRustFeaturesResult;
    const METHOD: &'static str = "buildTarget/disableRustFeatures";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisableRustFeaturesParams {
    pub package_id: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisableRustFeaturesResult {}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "example_feature";

    #[test]
    fn disable_rust_features_method() {
        assert_eq!(
            DisableRustFeatures::METHOD,
            "buildTarget/disableRustFeatures"
        );
    }

    #[test]
    fn enable_rust_features_params() {
        test_deserialization(
            r#"{"packageId": "package_id", "features":["example_feature"]}"#,
            &DisableRustFeaturesParams {
                package_id: PACKAGE_ID.into(),
                features: vec![FEATURE.into()],
            },
        );
        test_deserialization(
            r#"{"packageId": "","features":[]}"#,
            &DisableRustFeaturesParams::default(),
        );
    }

    #[test]
    fn disable_rust_features_result() {
        let test_data = DisableRustFeaturesResult {};
        assert_json_snapshot!(test_data, @"{}");
    }
}
