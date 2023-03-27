use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum Reload {}

impl Request for Reload {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "workspace/reload";
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn reload_method() {
        assert_eq!(Reload::METHOD, "workspace/reload");
    }

    #[test]
    fn reload_params() {
        test_deserialization(r#"null"#, &());
    }

    #[test]
    fn reload_result() {
        test_serialization(&(), r#"null"#);
    }
}
