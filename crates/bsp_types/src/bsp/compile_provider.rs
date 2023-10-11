use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileProvider {
    pub language_ids: Vec<LanguageId>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn compile_provider() {
        let test_data = CompileProvider {
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
        assert_json_snapshot!(CompileProvider::default(),
            @r#"
        {
          "languageIds": []
        }
        "#
        );
    }
}
