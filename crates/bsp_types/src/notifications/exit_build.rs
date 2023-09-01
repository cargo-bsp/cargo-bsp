use crate::notifications::Notification;

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
