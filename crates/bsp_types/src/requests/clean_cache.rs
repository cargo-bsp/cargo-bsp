use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::BuildTargetIdentifier;

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
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

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
        test_deserialization(r#"{"targets":[]}"#, &CleanCacheParams::default());
    }

    #[test]
    fn clean_cache_result() {
        let test_data = CleanCacheResult {
            message: Some("test_message".to_string()),
            cleaned: true,
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "message": "test_message",
          "cleaned": true
        }
        "###
        );
        assert_json_snapshot!(CleanCacheResult::default(),
            @r###"
        {
          "cleaned": false
        }
        "###
        );
    }
}
