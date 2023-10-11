use serde::{Deserialize, Serialize};

use crate::requests::Request;
use crate::BuildTargetIdentifier;

/// The clean cache request is sent from the client to the server to reset any state
/// associated with a given build target. The state can live either in the build
/// tool or in the file system.
///
/// The build tool defines the exact semantics of the clean cache request:
///
/// 1. Stateless build tools are free to ignore the request and respond with a
///    successful response.
/// 2. Stateful build tools must ensure that invoking compilation on a target that
///    has been cleaned results in a full compilation.
#[derive(Debug)]
pub enum BuildTargetCleanCache {}

impl Request for BuildTargetCleanCache {
    type Params = CleanCacheParams;
    type Result = CleanCacheResult;
    const METHOD: &'static str = "buildTarget/cleanCache";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheParams {
    /// The build targets to clean.
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanCacheResult {
    /// Optional message to display to the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Indicates whether the clean cache request was performed or not.
    pub cleaned: bool,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn clean_cache_method() {
        assert_eq!(BuildTargetCleanCache::METHOD, "buildTarget/cleanCache");
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
            @r#"
        {
          "message": "test_message",
          "cleaned": true
        }
        "#
        );
        assert_json_snapshot!(CleanCacheResult::default(),
            @r#"
        {
          "cleaned": false
        }
        "#
        );
    }
}
