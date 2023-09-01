use crate::requests::Request;

#[derive(Debug)]
pub enum WorkspaceReload {}

impl Request for WorkspaceReload {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "workspace/reload";
}

#[cfg(test)]
mod tests {
    use insta::assert_json_snapshot;

    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn reload_method() {
        assert_eq!(WorkspaceReload::METHOD, "workspace/reload");
    }

    #[test]
    fn reload_params() {
        test_deserialization(r#"null"#, &());
    }

    #[test]
    fn reload_result() {
        assert_json_snapshot!((), @"null");
    }
}
