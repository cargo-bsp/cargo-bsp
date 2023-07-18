use crate::requests::cargo_extension::Feature;
use serde::{Deserialize, Serialize};

use crate::requests::Request;

#[derive(Debug)]
pub enum DisableCargoFeatures {}

impl Request for DisableCargoFeatures {
    type Params = DisableCargoFeaturesParams;
    type Result = ();
    const METHOD: &'static str = "workspace/disableCargoFeatures";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisableCargoFeaturesParams {
    pub package_id: String,
    pub features: Vec<Feature>,
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "feature";

    #[test]
    fn disable_cargo_features_method() {
        assert_eq!(
            DisableCargoFeatures::METHOD,
            "workspace/disableCargoFeatures"
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
}
