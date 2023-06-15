use crate::requests::cargo_extension::Feature;
use serde::{Deserialize, Serialize};

use crate::requests::Request;

#[derive(Debug)]
pub enum EnableCargoFeatures {}

impl Request for EnableCargoFeatures {
    type Params = EnableCargoFeaturesParams;
    type Result = EnableCargoFeaturesResult;
    const METHOD: &'static str = "buildTarget/enableCargoFeatures";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnableCargoFeaturesParams {
    pub package_id: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EnableCargoFeaturesResult {}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "feature";

    #[test]
    fn enable_cargo_features_method() {
        assert_eq!(
            EnableCargoFeatures::METHOD,
            "buildTarget/enableCargoFeatures"
        );
    }

    #[test]
    fn enable_cargo_features_params() {
        test_deserialization(
            r#"{"packageId": "package_id", "features":["feature"]}"#,
            &EnableCargoFeaturesParams {
                package_id: PACKAGE_ID.into(),
                features: vec![FEATURE.into()],
            },
        );
        test_deserialization(
            r#"{"packageId": "","features":[]}"#,
            &EnableCargoFeaturesParams::default(),
        );
    }

    #[test]
    fn enable_cargo_features_result() {
        let test_data = EnableCargoFeaturesResult {};
        assert_json_snapshot!(test_data, @"{}");
    }
}
