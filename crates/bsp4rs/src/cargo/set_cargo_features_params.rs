use serde::{Deserialize, Serialize};

use crate::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetCargoFeaturesParams {
    /// Package ID for which new features state will be set.
    pub package_id: String,
    /// The list of features to be set as a new state.
    pub features: BTreeSet<Feature>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_deserialization;

    const PACKAGE_ID: &str = "package_id";
    const FEATURE: &str = "feature";

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
}
