use serde::{Deserialize, Serialize};

use crate::*;

/// The completion of a compilation task should be signalled with a
/// `build/taskFinish` notification. When the compilation unit is a build target,
/// the notification's `dataKind` field must be `compile-report` and the `data`
/// field must include a `CompileReport` object:
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileReport {
    /// The build target that was compiled.
    pub target: BuildTargetIdentifier,
    /// An optional request id to know the origin of this report.
    #[deprecated(note = "Use the field in TaskFinishParams instead")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_id: Option<Identifier>,
    /// The total number of reported errors compiling this target.
    pub errors: i32,
    /// The total number of reported warnings compiling the target.
    pub warnings: i32,
    /// The total number of milliseconds it took to compile the target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    /// The compilation was a noOp compilation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_op: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn compile_report() {
        #[allow(deprecated)]
        let test_data = CompileReport {
            target: BuildTargetIdentifier::default(),
            origin_id: Some("test_originId".into()),
            errors: 1,
            warnings: 2,
            time: Some(3),
            no_op: Some(true),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "originId": "test_originId",
          "errors": 1,
          "warnings": 2,
          "time": 3,
          "noOp": true
        }
        "#
        );
        assert_json_snapshot!(CompileReport::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "errors": 0,
          "warnings": 0
        }
        "#
        );
    }
}
