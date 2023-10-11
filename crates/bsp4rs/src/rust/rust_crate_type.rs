use serde_repr::{Deserialize_repr, Serialize_repr};

/// Crate types (`lib`, `rlib`, `dylib`, `cdylib`, `staticlib`) are listed for
/// `lib` and `example` target kinds. For other target kinds `bin` crate type is listed.
#[derive(
    Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum RustCrateType {
    #[default]
    Bin = 1,
    Lib = 2,
    Rlib = 3,
    Dylib = 4,
    Cdylib = 5,
    Staticlib = 6,
    ProcMacro = 7,
    Unknown = 8,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_crate_type() {
        assert_json_snapshot!(RustCrateType::Bin, @"1");
        assert_json_snapshot!(RustCrateType::Lib, @"2");
        assert_json_snapshot!(RustCrateType::Rlib, @"3");
        assert_json_snapshot!(RustCrateType::Dylib, @"4");
        assert_json_snapshot!(RustCrateType::Cdylib, @"5");
        assert_json_snapshot!(RustCrateType::Staticlib, @"6");
        assert_json_snapshot!(RustCrateType::ProcMacro, @"7");
        assert_json_snapshot!(RustCrateType::Unknown, @"8");
    }
}
