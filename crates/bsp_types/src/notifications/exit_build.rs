use crate::notifications::Notification;

#[derive(Debug)]
pub enum ExitBuild {}

impl Notification for ExitBuild {
    type Params = ();
    const METHOD: &'static str = "build/exit";
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn exit_build_method() {
        assert_eq!(ExitBuild::METHOD, "build/exit");
    }

    #[test]
    fn exit_build_params() {
        test_deserialization(r#"null"#, &());
    }
}
