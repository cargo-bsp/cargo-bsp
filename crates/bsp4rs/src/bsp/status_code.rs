use serde_repr::{Deserialize_repr, Serialize_repr};

/// Included in notifications of tasks or requests to signal the completion state.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum StatusCode {
    #[default]
    /// Execution was successful.
    Ok = 1,
    /// Execution failed.
    Error = 2,
    /// Execution was cancelled.
    Cancelled = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn status_code() {
        assert_json_snapshot!(StatusCode::Ok, @"1");
        assert_json_snapshot!(StatusCode::Error, @"2");
        assert_json_snapshot!(StatusCode::Cancelled, @"3");
    }
}
