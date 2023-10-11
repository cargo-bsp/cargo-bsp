use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencySourcesResult {
    pub items: Vec<DependencySourcesItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn dependency_sources_result() {
        let test_data = DependencySourcesResult {
            items: vec![DependencySourcesItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
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
        "#
        );
        assert_json_snapshot!(DependencySourcesResult::default(),
            @r#"
        {
          "items": []
        }
        "#
        );
    }
}
