use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::bsp_types::notifications::Notification;
use crate::bsp_types::BuildTargetIdentifier;

#[derive(Debug)]
pub enum DidChangeBuildTarget {}

impl Notification for DidChangeBuildTarget {
    type Params = DidChangeBuildTargetParams;
    const METHOD: &'static str = "buildTarget/didChange";
}

/* Build Target Changed Notification params */
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
    use crate::bsp_types::tests::test_serialization;

    use super::*;

    #[test]
    fn did_change_build_target_method() {
        assert_eq!(DidChangeBuildTarget::METHOD, "buildTarget/didChange");
    }

    #[test]
    fn did_change_build_target_params() {
        let test_data = DidChangeBuildTargetParams {
            changes: vec![BuildTargetEvent::default(), BuildTargetEvent::default()],
        };

        test_serialization(
            &test_data,
            r#"{"changes":[{"target":{"uri":""}},{"target":{"uri":""}}]}"#,
        );

        let mut modified_data = test_data.clone();
        modified_data.changes = vec![];
        test_serialization(&modified_data, r#"{"changes":[]}"#);
    }

    #[test]
    fn build_target_event() {
        let test_data = BuildTargetEvent {
            target: BuildTargetIdentifier::default(),
            kind: Some(BuildTargetEventKind::default()),
            data: Some(serde_json::json!({"dataKey": "dataValue"})),
        };

        test_serialization(
            &test_data,
            r#"{"target":{"uri":""},"kind":1,"data":{"dataKey":"dataValue"}}"#,
        );

        let mut modified_data = test_data.clone();
        modified_data.kind = None;
        test_serialization(
            &modified_data,
            r#"{"target":{"uri":""},"data":{"dataKey":"dataValue"}}"#,
        );
        modified_data = test_data.clone();
        modified_data.data = None;
        test_serialization(&modified_data, r#"{"target":{"uri":""},"kind":1}"#);
    }

    #[test]
    fn build_target_event_kind() {
        test_serialization(&BuildTargetEventKind::Created, r#"1"#);
        test_serialization(&BuildTargetEventKind::Changed, r#"2"#);
        test_serialization(&BuildTargetEventKind::Deleted, r#"3"#);
    }
}
