use serde::{Deserialize, Serialize};

use crate::extensions::RequestId;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelRequestParams {
    /// The request id to cancel.
    pub id: RequestId,
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;

    use super::*;

    #[test]
    fn cancel_params() {
        test_deserialization(
            r#"{"id":123}"#,
            &CancelRequestParams {
                id: RequestId::I32(123),
            },
        );
        test_deserialization(
            r#"{"id":"test_id"}"#,
            &CancelRequestParams {
                id: RequestId::String("test_id".to_string()),
            },
        );
    }
}
