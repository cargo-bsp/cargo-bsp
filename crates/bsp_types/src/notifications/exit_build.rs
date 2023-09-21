use crate::notifications::Notification;

/// Like the language server protocol, a notification to ask the server to exit its process. The server should exit with success code 0
/// if the shutdown request has been received before; otherwise with error code 1.
#[derive(Debug)]
pub enum OnBuildExit {}

impl Notification for OnBuildExit {
    type Params = ();
    const METHOD: &'static str = "build/exit";
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn exit_build_method() {
        assert_eq!(OnBuildExit::METHOD, "build/exit");
    }

    #[test]
    fn exit_build_params() {
        test_deserialization(r#"null"#, &());
    }
}
