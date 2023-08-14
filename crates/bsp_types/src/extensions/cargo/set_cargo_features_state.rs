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
    pub package_id: String,
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
    use insta::assert_json_snapshot;

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
        let test_data = SetCargoFeaturesResult {
            status_code: StatusCode::Ok,
        };
        assert_json_snapshot!(test_data, @r###"
        {
          "statusCode": 1
        }
        "###);
    }
}
