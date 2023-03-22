use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum ShutdownBuild {}

impl Request for ShutdownBuild {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "build/shutdown";
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::{test_deserialization, test_serialization};

    use super::*;

    #[test]
    fn shutdown_build_method() {
        assert_eq!(ShutdownBuild::METHOD, "build/shutdown");
    }

    #[test]
    fn shutdown_build_params() {
        test_deserialization(r#"null"#, &());
    }

    #[test]
    fn shutdown_build_result() {
        test_serialization(&(), r#"null"#);
    }
}
