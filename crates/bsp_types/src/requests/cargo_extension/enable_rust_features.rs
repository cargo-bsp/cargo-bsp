use crate::requests::cargo_extension::Feature;
use serde::{Deserialize, Serialize};

use crate::requests::Request;

#[derive(Debug)]
pub enum EnableRustFeatures {}

impl Request for EnableRustFeatures {
    type Params = EnableRustFeaturesParams;
    type Result = EnableRustFeaturesResult;
    const METHOD: &'static str = "buildTarget/enableRustFeatures";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnableRustFeaturesParams {
    pub package_id: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnableRustFeaturesResult {}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "example_feature";

    #[test]
    fn enable_rust_features_method() {
        assert_eq!(EnableRustFeatures::METHOD, "buildTarget/enableRustFeatures");
    }

    #[test]
    fn enable_rust_features_params() {
        test_deserialization(
            r#"{"packageId": "package_id", "features":["example_feature"]}"#,
            &EnableRustFeaturesParams {
                package_id: PACKAGE_ID.into(),
                features: vec![FEATURE.into()],
            },
        );
        test_deserialization(
            r#"{"packageId": "","features":[]}"#,
            &EnableRustFeaturesParams::default(),
        );
    }

    #[test]
    fn enable_rust_features_result() {
        let test_data = EnableRustFeaturesResult {};
        assert_json_snapshot!(test_data, @"{}");
    }
}
