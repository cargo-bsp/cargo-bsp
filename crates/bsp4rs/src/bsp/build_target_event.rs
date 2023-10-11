use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetEvent {
    /// The identifier for the changed build target
    pub target: BuildTargetIdentifier,
    /// The kind of change for this build target
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<BuildTargetEventKind>,
    /// Any additional metadata about what information changed.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<BuildTargetEventData>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn build_target_event() {
        let test_data = BuildTargetEvent {
            target: BuildTargetIdentifier::default(),
            kind: Some(BuildTargetEventKind::default()),
            data: Some(BuildTargetEventData::Other(OtherData {
                data_kind: "test_dataKind".to_string(),
                data: serde_json::json!({"dataKey": "dataValue"}),
            })),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "kind": 1,
          "dataKind": "test_dataKind",
          "data": {
            "dataKey": "dataValue"
          }
        }
        "#
        );
        assert_json_snapshot!(BuildTargetEvent::default(),
            @r#"
        {
          "target": {
            "uri": ""
          }
        }
        "#
        );
    }
}
