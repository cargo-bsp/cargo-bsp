use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum Resources {}

impl Request for Resources {
    type Params = ResourcesParams;
    type Result = ResourcesResult;
    const METHOD: &'static str = "buildTarget/resources";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourcesResult {
    pub items: Vec<ResourcesItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct ResourcesItem {
    pub target: BuildTargetIdentifier,
    /** List of resource files. */
    pub resources: Vec<Uri>,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn resources_method() {
        assert_eq!(Resources::METHOD, "buildTarget/resources");
    }

    #[test]
    fn resources_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &ResourcesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &ResourcesParams { targets: vec![] });
    }

    #[test]
    fn resources_result() {
        test_serialization(
            &ResourcesResult {
                items: vec![ResourcesItem::default()],
            },
            r#"{"items":[{"target":{"uri":""},"resources":[]}]}"#,
        );
        test_serialization(&ResourcesResult { items: vec![] }, r#"{"items":[]}"#);
    }

    #[test]
    fn resources_item() {
        let test_data = ResourcesItem {
            target: BuildTargetIdentifier::default(),
            resources: vec![Uri::default()],
        };

        test_serialization(&test_data, r#"{"target":{"uri":""},"resources":[""]}"#);

        let mut modified = test_data;
        modified.resources = vec![];
        test_serialization(&modified, r#"{"target":{"uri":""},"resources":[]}"#);
    }
}
