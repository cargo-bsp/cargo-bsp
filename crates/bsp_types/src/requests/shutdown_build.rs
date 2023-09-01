use crate::requests::Request;

#[derive(Debug)]
pub enum BuildShutdown {}

impl Request for BuildShutdown {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "build/shutdown";
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn shutdown_build_method() {
        assert_eq!(BuildShutdown::METHOD, "build/shutdown");
    }

    #[test]
    fn shutdown_build_params() {
        test_deserialization(r#"null"#, &());
    }

    #[test]
    fn shutdown_build_result() {
        assert_json_snapshot!((), @"null");
    }
}
