use serde::{Deserialize, Serialize};

use crate::bsp_types::requests::Request;
use crate::bsp_types::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum JavaExtensions {}

impl Request for JavaExtensions {
    type Params = JavacOptionsParams;
    type Result = JavacOptionsResult;
    const METHOD: &'static str = "buildTarget/javacOptions";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct JavacOptionsParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct JavacOptionsResult {
    pub items: Vec<JavacOptionsItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JavacOptionsItem {
    pub target: BuildTargetIdentifier,

    /** Additional arguments to the compiler.
     * For example, -deprecation. */
    options: Vec<String>,

    /** The dependency classpath for this target, must be
     * identical to what is passed as arguments to
     * the -classpath flag in the command line interface
     * of javac. */
    classpath: Vec<Uri>,

    /** The output directory for classfiles produced by this target */
    class_directory: Uri,
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn java_extensions_method() {
        assert_eq!(JavaExtensions::METHOD, "buildTarget/javacOptions");
    }

    #[test]
    fn javac_options_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &JavacOptionsParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &JavacOptionsParams { targets: vec![] });
    }

    #[test]
    fn javac_options_result() {
        test_serialization(
            &JavacOptionsResult {
                items: vec![JavacOptionsItem::default()],
            },
            r#"{"items":[{"target":{"uri":""},"options":[],"classpath":[],"classDirectory":""}]}"#,
        );
        test_serialization(&JavacOptionsResult { items: vec![] }, r#"{"items":[]}"#);
    }

    #[test]
    fn javac_options_item() {
        let test_data = JavacOptionsItem {
            target: BuildTargetIdentifier::default(),
            options: vec!["test_options".to_string()],
            classpath: vec![Uri::default()],
            class_directory: Uri::default(),
        };

        test_serialization(
            &test_data,
            r#"{"target":{"uri":""},"options":["test_options"],"classpath":[""],"classDirectory":""}"#,
        );

        let mut modified = test_data.clone();
        modified.options = vec![];
        test_serialization(
            &modified,
            r#"{"target":{"uri":""},"options":[],"classpath":[""],"classDirectory":""}"#,
        );
        modified = test_data.clone();
        modified.classpath = vec![];
        test_serialization(
            &modified,
            r#"{"target":{"uri":""},"options":["test_options"],"classpath":[],"classDirectory":""}"#,
        );
    }
}
