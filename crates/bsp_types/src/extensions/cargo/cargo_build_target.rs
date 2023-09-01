use serde::{Deserialize, Serialize};

/** `CargoBuildTarget` is a basic data structure that contains
cargo-specific metadata. */
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoBuildTarget {
    pub edition: Edition,
    pub required_features: Vec<String>,
}

/** The Rust edition.
As of writing this comment rust editions 2024, 2027 and 2030 are not
actually a thing yet but are parsed nonetheless for future proofing. */
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Edition(pub std::borrow::Cow<'static, str>);
impl Edition {
    pub const E2015: Edition = Edition::new("2015");
    pub const E2018: Edition = Edition::new("2018");
    pub const E2021: Edition = Edition::new("2021");

    pub const fn new(tag: &'static str) -> Self {
        Edition(std::borrow::Cow::Borrowed(tag))
    }
}

#[cfg(test)]
mod tests {
    use crate::extensions::Edition;
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn cargo_build_target() {
        let test_data = CargoBuildTarget {
            edition: Edition::E2015,
            required_features: vec!["test_requiredFeature".to_string()],
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "edition": "2015",
          "requiredFeatures": [
            "test_requiredFeature"
          ]
        }
        "#
        );
        assert_json_snapshot!(CargoBuildTarget::default(),
            @r#"
        {
          "edition": "",
          "requiredFeatures": []
        }
        "#
        );
    }
}
