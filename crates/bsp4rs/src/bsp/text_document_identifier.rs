use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentIdentifier {
    /// The text document's URI.
    pub uri: URI,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn text_document_identifier() {
        let test_data = TextDocumentIdentifier {
            uri: "test_uri".into(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "uri": "test_uri"
        }
        "#
        );
        assert_json_snapshot!(TextDocumentIdentifier::default(),
            @r#"
        {
          "uri": ""
        }
        "#
        );
    }
}
