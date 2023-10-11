use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyModulesResult {
    pub items: Vec<DependencyModulesItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn dependency_modules_result() {
        let test_data = DependencyModulesResult {
            items: vec![DependencyModulesItem::default()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "items": [
            {
              "target": {
                "uri": ""
              },
              "modules": []
            }
          ]
        }
        "#
        );
        assert_json_snapshot!(DependencyModulesResult::default(),
            @r#"
        {
          "items": []
        }
        "#
        );
    }
}
