use super::Feature;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::requests::Request;
use crate::StatusCode;

#[derive(Debug)]
pub enum SetCargoFeatures {}

impl Request for SetCargoFeatures {
    type Params = SetCargoFeaturesParams;
    type Result = SetCargoFeaturesResult;
    const METHOD: &'static str = "workspace/setCargoFeatures";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SetCargoFeaturesParams {
    /** Id of a package the features we want to set*/
    pub package_id: String,
    //** A list of features the server is supposed to set */
    pub features: BTreeSet<Feature>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SetCargoFeaturesResult {
    pub status_code: StatusCode,
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "feature";

    #[test]
    fn set_cargo_features_method() {
        assert_eq!(SetCargoFeatures::METHOD, "workspace/setCargoFeatures");
    }

    #[test]
    fn set_cargo_features_params() {
        test_deserialization(
            r#"{"packageId": "package_id", "features":["feature"]}"#,
            &SetCargoFeaturesParams {
                package_id: PACKAGE_ID.into(),
                features: BTreeSet::from([FEATURE.into()]),
            },
        );
        test_deserialization(
            r#"{"packageId": "","features":[]}"#,
            &SetCargoFeaturesParams::default(),
        );
    }
    #[test]
    fn set_cargo_features_result() {
        test_deserialization(
            r#"{"statusCode": 1}"#,
            &SetCargoFeaturesResult {
                status_code: StatusCode::Ok,
            },
        );
    }
}
