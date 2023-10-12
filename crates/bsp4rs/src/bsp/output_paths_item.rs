use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputPathsItem {
    /// A build target to which output paths item belongs.
    pub target: BuildTargetIdentifier,
    /// Output paths.
    pub output_paths: Vec<OutputPathItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

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
}
