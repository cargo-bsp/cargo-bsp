use serde::{Deserialize, Serialize};

use crate::notifications::Notification;

#[derive(Debug)]
pub enum InitializedBuild {}

impl Notification for InitializedBuild {
    type Params = InitializedBuildParams;
    const METHOD: &'static str = "build/initialized";
}

/** Initialized Build notification params */
#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct InitializedBuildParams {}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn initialized_build_method() {
        assert_eq!(InitializedBuild::METHOD, "build/initialized");
    }

    #[test]
    fn initialized_build_params() {
        test_deserialization(r#"{}"#, &InitializedBuildParams {});
    }
}
