use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::Notification;
use crate::BuildTargetIdentifier;

#[derive(Debug)]
pub enum DidChangeBuildTarget {}

impl Notification for DidChangeBuildTarget {
    type Params = DidChangeBuildTargetParams;
    const METHOD: &'static str = "buildTarget/didChange";
}

/**Build Target Changed Notification params */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct DidChangeBuildTargetParams {
    pub changes: Vec<BuildTargetEvent>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct BuildTargetEvent {
    /** The identifier for the changed build target */
    pub target: BuildTargetIdentifier,

    /** The kind of change for this build target */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<BuildTargetEventKind>,

    /** Any additional metadata about what information changed. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum BuildTargetEventKind {
    /** The build target is new. */
    #[default]
    Created = 1,
    /** The build target has changed. */
    Changed = 2,
    /** The build target has been deleted. */
    Deleted = 3,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn did_change_build_target_method() {
        assert_eq!(DidChangeBuildTarget::METHOD, "buildTarget/didChange");
    }

    #[test]
    fn did_change_build_target_params() {
        let test_data = DidChangeBuildTargetParams {
            changes: vec![BuildTargetEvent::default()],
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "changes": [
            {
              "target": {
                "uri": ""
              }
            }
          ]
        }
        "###
        );
        assert_json_snapshot!(DidChangeBuildTargetParams::default(),
            @r###"
        {
          "changes": []
        }
        "###
        );
    }

    #[test]
    fn build_target_event() {
        let test_data = BuildTargetEvent {
            target: BuildTargetIdentifier::default(),
            kind: Some(BuildTargetEventKind::default()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "target": {
            "uri": ""
          },
          "kind": 1,
          "data": {
            "dataKey": "dataValue"
          }
        }
        "###
        );
        assert_json_snapshot!(BuildTargetEvent::default(),
            @r###"
        {
          "target": {
            "uri": ""
          }
        }
        "###
        );
    }

    #[test]
    fn build_target_event_kind() {
        assert_json_snapshot!(BuildTargetEventKind::Created, @"1");
        assert_json_snapshot!(BuildTargetEventKind::Changed, @"2");
        assert_json_snapshot!(BuildTargetEventKind::Deleted, @"3");
    }
}
