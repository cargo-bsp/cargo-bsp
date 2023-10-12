use serde::{Deserialize, Serialize};

/// Clients can use these capabilities to notify users what BSP endpoints can and
/// cannot be used and why.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetCapabilities {
    /// This target can be compiled by the BSP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_compile: Option<bool>,
    /// This target can be tested by the BSP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_test: Option<bool>,
    /// This target can be run by the BSP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_run: Option<bool>,
    /// This target can be debugged by the BSP server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_debug: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn build_target_capabilities() {
        let test_data = BuildTargetCapabilities {
            can_compile: Some(true),
            can_test: Some(true),
            can_run: Some(true),
            can_debug: Some(true),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "canCompile": true,
          "canTest": true,
          "canRun": true,
          "canDebug": true
        }
        "#
        );
        assert_json_snapshot!(BuildTargetCapabilities::default(),
            @r#"
        {}
        "#
        );
    }
}
