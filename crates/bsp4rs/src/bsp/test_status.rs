use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum TestStatus {
    #[default]
    /// The test passed successfully.
    Passed = 1,
    /// The test failed.
    Failed = 2,
    /// The test was marked as ignored.
    Ignored = 3,
    /// The test execution was cancelled.
    Cancelled = 4,
    /// The was not included in execution.
    Skipped = 5,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn test_status() {
        assert_json_snapshot!(TestStatus::Passed, @"1");
        assert_json_snapshot!(TestStatus::Failed, @"2");
        assert_json_snapshot!(TestStatus::Ignored, @"3");
        assert_json_snapshot!(TestStatus::Cancelled, @"4");
        assert_json_snapshot!(TestStatus::Skipped, @"5");
    }
}
