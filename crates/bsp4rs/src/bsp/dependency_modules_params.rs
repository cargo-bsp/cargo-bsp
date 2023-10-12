use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModulesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_deserialization;

    #[test]
    fn dependency_modules_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &DependencyModulesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &DependencyModulesParams::default());
    }
}
