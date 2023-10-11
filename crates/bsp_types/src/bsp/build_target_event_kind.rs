use serde_repr::{Deserialize_repr, Serialize_repr};

/// The `BuildTargetEventKind` information can be used by clients to trigger
/// reindexing or update the user interface with the new information.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum BuildTargetEventKind {
    #[default]
    /// The build target is new.
    Created = 1,
    /// The build target has changed.
    Changed = 2,
    /// The build target has been deleted.
    Deleted = 3,
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn build_target_event_kind() {
        assert_json_snapshot!(BuildTargetEventKind::Created, @"1");
        assert_json_snapshot!(BuildTargetEventKind::Changed, @"2");
        assert_json_snapshot!(BuildTargetEventKind::Deleted, @"3");
    }
}
