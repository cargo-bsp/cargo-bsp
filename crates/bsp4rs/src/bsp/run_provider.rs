use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunProvider {
    pub language_ids: Vec<LanguageId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn run_provider() {
        let test_data = RunProvider {
            language_ids: vec!["test_languageId".into()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "languageIds": [
            "test_languageId"
          ]
        }
        "#
        );
        assert_json_snapshot!(RunProvider::default(),
            @r#"
        {
          "languageIds": []
        }
        "#
        );
    }
}
