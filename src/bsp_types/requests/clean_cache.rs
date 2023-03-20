use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::BuildTargetIdentifier;

#[derive(Debug)]
pub enum CleanCache {}

impl Request for CleanCache {
    type Params = CleanCacheParams;
    type Result = CleanCacheResult;
    const METHOD: &'static str = "buildTarget/cleanCache";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct CleanCacheParams {
    /** The build targets to clean. */
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct CleanCacheResult {
    /** Optional message to display to the user. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /** Indicates whether the clean cache request was performed or not. */
    pub cleaned: bool,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn clean_cache_method() {
        assert_eq!(CleanCache::METHOD, "buildTarget/cleanCache");
    }

    #[test]
    fn clean_cache_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &CleanCacheParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &CleanCacheParams { targets: vec![] });
    }

    #[test]
    fn clean_cache_result() {
        let test_data = CleanCacheResult {
            message: Some("test_message".to_string()),
            cleaned: true,
        };

        test_serialization(&test_data, r#"{"message":"test_message","cleaned":true}"#);

        let mut modified = test_data.clone();
        modified.message = None;
        test_serialization(&modified, r#"{"cleaned":true}"#);
    }
}
