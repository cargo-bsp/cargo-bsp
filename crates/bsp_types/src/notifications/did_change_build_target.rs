use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::notifications::Notification;
use crate::{BuildTargetIdentifier, OtherData};

/// The build target changed notification is sent from the server to the client to
/// signal a change in a build target. The server communicates during the initialize
/// handshake whether this method is supported or not.
#[derive(Debug)]
pub enum OnBuildTargetDidChange {}

impl Notification for OnBuildTargetDidChange {
    type Params = DidChangeBuildTarget;
    const METHOD: &'static str = "buildTarget/didChange";
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeBuildTarget {
    pub changes: Vec<BuildTargetEvent>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetEvent {
    /// The identifier for the changed build target
    pub target: BuildTargetIdentifier,
    /// The kind of change for this build target
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<BuildTargetEventKind>,
    /// Any additional metadata about what information changed.
    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub data: Option<BuildTargetEventData>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedBuildTargetEventData {}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BuildTargetEventData {
    Named(NamedBuildTargetEventData),
    Other(OtherData),
}

impl BuildTargetEventData {}

/// The `BuildTargetEventKind` information can be used by clients to trigger
/// reindexing or update the user interface with the new information.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum BuildTargetEventKind {
    #[default]
    /// The build target is new.
    Created = 1,
    /// The build target has changed.
    Changed = 2,
    /// The build target has been deleted.
    Deleted = 3,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn did_change_build_target_method() {
        assert_eq!(OnBuildTargetDidChange::METHOD, "buildTarget/didChange");
    }

    #[test]
    fn did_change_build_target_params() {
        let test_data = DidChangeBuildTarget {
            changes: vec![BuildTargetEvent::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "changes": [
            {
              "target": {
                "uri": ""
              }
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(DidChangeBuildTarget::default(),
            @r#"
        {
          "changes": []
        }
        "#
        );
    }

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

    #[test]
    fn build_target_event_kind() {
        assert_json_snapshot!(BuildTargetEventKind::Created, @"1");
        assert_json_snapshot!(BuildTargetEventKind::Changed, @"2");
        assert_json_snapshot!(BuildTargetEventKind::Deleted, @"3");
    }
}
