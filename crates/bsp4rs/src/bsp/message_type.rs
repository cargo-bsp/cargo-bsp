use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum MessageType {
    #[default]
    /// An error message.
    Error = 1,
    /// A warning message.
    Warning = 2,
    /// An information message.
    Info = 3,
    /// A log message.
    Log = 4,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn message_type() {
        assert_json_snapshot!(MessageType::Error, @"1");
        assert_json_snapshot!(MessageType::Warning, @"2");
        assert_json_snapshot!(MessageType::Info, @"3");
        assert_json_snapshot!(MessageType::Log, @"4");
    }
}
