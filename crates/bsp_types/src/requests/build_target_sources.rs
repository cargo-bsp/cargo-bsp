use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::requests::Request;
use crate::{BuildTargetIdentifier, Uri};

#[derive(Debug)]
pub enum Sources {}

impl Request for Sources {
    type Params = SourcesParams;
    type Result = SourcesResult;
    const METHOD: &'static str = "buildTarget/sources";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SourcesParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SourcesResult {
    pub items: Vec<SourcesItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SourcesItem {
    pub target: BuildTargetIdentifier,

    /** The text documents or and directories that belong to this build target. */
    sources: Vec<SourceItem>,

    /** The root directories from where source files should be relativized.
    Example: ["file://Users/name/dev/metals/src/main/scala"] */
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    roots: Vec<Uri>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SourceItem {
    /** Either a text document or a directory. A directory entry must end with a forward
    slash "/" and a directory entry implies that every nested text document within the
    directory belongs to this source item. */
    pub uri: Uri,

    /** Type of file of the source item, such as whether it is file or directory. */
    pub kind: SourceItemKind,

    /** Indicates if this source is automatically generated by the build and is not
    intended to be manually edited by the user. */
    pub generated: bool,
}

#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum SourceItemKind {
    /** The source item references a normal file.  */
    #[default]
    File = 1,
    /** The source item references a directory. */
    Directory = 2,
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn sources_method() {
        assert_eq!(Sources::METHOD, "buildTarget/sources");
    }

    #[test]
    fn sources_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &SourcesParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &SourcesParams::default());
    }

    #[test]
    fn sources_result() {
        let test_data = SourcesResult {
            items: vec![SourcesItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "items": [
            {
              "target": {
                "uri": ""
              },
              "sources": []
            }
          ]
        }
        "###
        );
        assert_json_snapshot!(SourcesResult::default(),
            @r###"
        {
          "items": []
        }
        "###
        );
    }

    #[test]
    fn sources_item() {
        let test_data = SourcesItem {
            target: BuildTargetIdentifier::default(),
            sources: vec![SourceItem::default()],
            roots: vec![Uri::default()],
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "target": {
            "uri": ""
          },
          "sources": [
            {
              "uri": "",
              "kind": 1,
              "generated": false
            }
          ],
          "roots": [
            ""
          ]
        }
        "###
        );
        assert_json_snapshot!(SourcesItem::default(),
            @r###"
        {
          "target": {
            "uri": ""
          },
          "sources": []
        }
        "###
        );
    }

    #[test]
    fn source_item() {
        let test_data = SourceItem {
            uri: "test_uri".into(),
            kind: SourceItemKind::File,
            generated: true,
        };

        assert_json_snapshot!(test_data,
            @r###"
        {
          "uri": "test_uri",
          "kind": 1,
          "generated": true
        }
        "###
        );
        assert_json_snapshot!(SourceItem::default(),
            @r###"
        {
          "uri": "",
          "kind": 1,
          "generated": false
        }
        "###
        );
    }

    #[test]
    fn source_item_kind() {
        assert_json_snapshot!(SourceItemKind::File, @"1");
        assert_json_snapshot!(SourceItemKind::Directory, @"2");
    }
}