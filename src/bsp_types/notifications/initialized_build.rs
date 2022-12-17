use serde::{Deserialize, Serialize};

use crate::bsp_types::notifications::Notification;

#[derive(Debug)]
pub enum InitializedBuild {}

impl Notification for InitializedBuild {
    type Params = InitializedBuildParams;
    const METHOD: &'static str = "build/initialized";
}

/* Initialized Build notification params */
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InitializedBuildParams {}
