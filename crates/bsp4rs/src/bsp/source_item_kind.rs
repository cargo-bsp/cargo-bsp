use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum SourceItemKind {
    #[default]
    /// The source item references a normal file.
    File = 1,
    /// The source item references a directory.
    Directory = 2,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn source_item_kind() {
        assert_json_snapshot!(SourceItemKind::File, @"1");
        assert_json_snapshot!(SourceItemKind::Directory, @"2");
    }
}
