use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum RustTargetKind {
    #[default]
    /// For lib targets.
    Lib = 1,
    /// For binaries.
    Bin = 2,
    /// For integration tests.
    Test = 3,
    /// For examples.
    Example = 4,
    /// For benchmarks.
    Bench = 5,
    /// For build scripts.
    CustomBuild = 6,
    /// For unknown targets.
    Unknown = 7,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_target_kind() {
        assert_json_snapshot!(RustTargetKind::Lib, @"1");
        assert_json_snapshot!(RustTargetKind::Bin, @"2");
        assert_json_snapshot!(RustTargetKind::Test, @"3");
        assert_json_snapshot!(RustTargetKind::Example, @"4");
        assert_json_snapshot!(RustTargetKind::Bench, @"5");
        assert_json_snapshot!(RustTargetKind::CustomBuild, @"6");
        assert_json_snapshot!(RustTargetKind::Unknown, @"7");
    }
}
