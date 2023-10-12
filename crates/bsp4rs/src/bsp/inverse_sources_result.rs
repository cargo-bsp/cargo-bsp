use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InverseSourcesResult {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn inverse_sources_result() {
        let test_data = InverseSourcesResult {
            targets: vec![BuildTargetIdentifier::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "targets": [
            {
              "uri": ""
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(InverseSourcesResult::default(),
            @r#"
        {
          "targets": []
        }
        "#
        );
    }
}
