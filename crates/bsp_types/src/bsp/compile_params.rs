use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileParams {
    /// A sequence of build targets to compile.
    pub targets: Vec<BuildTargetIdentifier>,
    /// A unique identifier generated by the client to identify this request.
    /// The server may include this id in triggered notifications or responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// Optional arguments to the compilation process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_deserialization;

    #[test]
    fn compile_params() {
        let test_data = CompileParams {
            targets: vec![BuildTargetIdentifier::default()],
            origin_id: Some("test_message".into()),
            arguments: Some(vec!["test_argument".to_string()]),
        };

        test_deserialization(
            r#"{"targets":[{"uri":""}],"originId":"test_message","arguments":["test_argument"]}"#,
            &test_data,
        );

        test_deserialization(r#"{"targets":[]}"#, &CompileParams::default());
    }
}
