use crate::requests::Request;

#[derive(Debug)]
pub enum BuildShutdown {}

/// Like the language server protocol, the shutdown build request is sent from the
/// client to the server. It asks the server to shut down, but to not exit
/// (otherwise the response might not be delivered correctly to the client). There
/// is a separate exit notification that asks the server to exit.
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
