use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum DependencySources {}

impl Request for DependencySources {
    type Params = DependencySourcesParams;
    type Result = DependencySourcesResult;
    const METHOD: &'static str = "buildTarget/dependencySources";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DependencySourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DependencySourcesResult {
    pub items: Vec<DependencySourcesItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct DependencySourcesItem {
    pub target: BuildTargetIdentifier,
    /** List of resources containing source files of the
     * target's dependencies.
     * Can be source files, jar files, zip files, or directories. */
    pub sources: Vec<Uri>,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn dependency_sources_method() {
        assert_eq!(DependencySources::METHOD, "buildTarget/dependencySources");
    }

    #[test]
    fn dependency_sources_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &DependencySourcesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(
            r#"{"targets":[]}"#,
            &DependencySourcesParams { targets: vec![] },
        );
    }

    #[test]
    fn dependency_sources_result() {
        test_serialization(
            &DependencySourcesResult {
                items: vec![DependencySourcesItem::default()],
            },
            r#"{"items":[{"target":{"uri":""},"sources":[]}]}"#,
        );
        test_serialization(
            &DependencySourcesResult { items: vec![] },
            r#"{"items":[]}"#,
        );
    }

    #[test]
    fn dependency_sources_item() {
        let test_data = DependencySourcesItem {
            target: BuildTargetIdentifier::default(),
            sources: vec![Uri::default()],
        };

        test_serialization(&test_data, r#"{"target":{"uri":""},"sources":[""]}"#);

        let mut modified = test_data;
        modified.sources = vec![];
        test_serialization(&modified, r#"{"target":{"uri":""},"sources":[]}"#);
    }
}
