use crate::bsp_types::notifications::Notification;

#[derive(Debug)]
pub enum ExitBuild {}

impl Notification for ExitBuild {
    type Params = ();
    const METHOD: &'static str = "build/exit";
}

#[cfg(test)]
mod tests {
    use crate::bsp_types::tests::test_serialization;

    use super::*;

    #[test]
    fn exit_build_method() {
        assert_eq!(ExitBuild::METHOD, "build/exit");
    }

    #[test]
    fn exit_build_params() {
        test_serialization(&(), r#"null"#);
    }
}
