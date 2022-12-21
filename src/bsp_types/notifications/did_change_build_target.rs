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
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeBuildTargetParams {
    pub changes: Vec<BuildTargetEvent>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize_repr, Deserialize_repr, Default)]
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
