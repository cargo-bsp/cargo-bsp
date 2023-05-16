use crate::notifications::Notification;

impl Notification for lsp_types::notification::Cancel {
    type Params = lsp_types::CancelParams;
    const METHOD: &'static str = "$/cancelRequest";
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn cancel() {
        assert_eq!(lsp_types::notification::Cancel::METHOD, "$/cancelRequest");
    }

    #[test]
    fn cancel_params() {
        test_deserialization(
            r#"{"id":123}"#,
            &lsp_types::CancelParams {
                id: lsp_types::NumberOrString::Number(123),
            },
        );
        test_deserialization(
            r#"{"id":"test_id"}"#,
            &lsp_types::CancelParams {
                id: lsp_types::NumberOrString::String("test_id".to_string()),
            },
        );
    }
}
