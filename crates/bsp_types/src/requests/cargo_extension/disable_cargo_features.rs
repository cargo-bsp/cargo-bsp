use crate::requests::cargo_extension::Feature;
use serde::{Deserialize, Serialize};

use crate::requests::Request;

#[derive(Debug)]
pub enum DisableCargoFeatures {}

impl Request for DisableCargoFeatures {
    type Params = DisableCargoFeaturesParams;
    type Result = DisableCargoFeaturesResult;
    const METHOD: &'static str = "buildTarget/disableCargoFeatures";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisableCargoFeaturesParams {
    pub package_id: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisableCargoFeaturesResult {}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "feature";

    #[test]
    fn disable_cargo_features_method() {
        assert_eq!(
            DisableCargoFeatures::METHOD,
            "buildTarget/disableCargoFeatures"
        );
    }

    #[test]
    fn enable_cargo_features_params() {
        test_deserialization(
            r#"{"packageId": "package_id", "features":["feature"]}"#,
            &DisableCargoFeaturesParams {
                package_id: PACKAGE_ID.into(),
                features: vec![FEATURE.into()],
            },
        );
        test_deserialization(
            r#"{"packageId": "","features":[]}"#,
            &DisableCargoFeaturesParams::default(),
        );
    }

    #[test]
    fn disable_cargo_features_result() {
        let test_data = DisableCargoFeaturesResult {};
        assert_json_snapshot!(test_data, @"{}");
    }
}
