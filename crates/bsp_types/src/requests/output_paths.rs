use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, URI};

#[derive(Debug)]
pub enum BuildTargetOutputPaths {}

impl Request for BuildTargetOutputPaths {
    type Params = OutputPathsParams;
    type Result = OutputPathsResult;
    const METHOD: &'static str = "buildTarget/outputPaths";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct OutputPathsParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct OutputPathsResult {
    pub items: Vec<OutputPathsItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathsItem {
    /** A build target to which output paths item belongs. */
    pub target: BuildTargetIdentifier,
    /** Output paths. */
    pub output_paths: Vec<OutputPathItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct OutputPathItem {
    /** Either a file or a directory. A directory entry must end with a forward
    slash "/" and a directory entry implies that every nested path within the
    directory belongs to this output item. */
    pub uri: URI,

    /** Type of file of the output item, such as whether it is file or directory. */
    pub kind: OutputPathItemKind,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum OutputPathItemKind {
    /** The output path item references a normal file. */
    #[default]
    File = 1,
    /** The output path item references a directory. */
    Directory = 2,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn output_paths_method() {
        assert_eq!(BuildTargetOutputPaths::METHOD, "buildTarget/outputPaths");
    }

    #[test]
    fn output_paths_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &OutputPathsParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &OutputPathsParams::default());
    }

    #[test]
    fn output_paths_result() {
        let test_data = OutputPathsResult {
            items: vec![OutputPathsItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "items": [
            {
              "target": {
                "uri": ""
              },
              "outputPaths": []
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(OutputPathsResult::default(),
            @r#"
        {
          "items": []
        }
        "#
        );
    }

    #[test]
    fn output_paths_item() {
        let test_data = OutputPathsItem {
            target: BuildTargetIdentifier::default(),
            output_paths: vec![OutputPathItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "target": {
            "uri": ""
          },
          "outputPaths": [
            {
              "uri": "",
              "kind": 1
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(OutputPathsItem::default(),
            @r#"
        {
          "target": {
            "uri": ""
          },
          "outputPaths": []
        }
        "#
        );
    }

    #[test]
    fn output_path_item() {
        let test_data = OutputPathItem {
            uri: "test_uri".into(),
            kind: OutputPathItemKind::File,
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "uri": "test_uri",
          "kind": 1
        }
        "#
        );
        assert_json_snapshot!(OutputPathItem::default(),
            @r#"
        {
          "uri": "",
          "kind": 1
        }
        "#
        );
    }

    #[test]
    fn status_code() {
        assert_json_snapshot!(OutputPathItemKind::File, @"1");
        assert_json_snapshot!(OutputPathItemKind::Directory, @"2");
    }
}
