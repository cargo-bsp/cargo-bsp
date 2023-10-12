use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustPackageOrigin(pub std::borrow::Cow<'static, str>);

impl RustPackageOrigin {
    /// External dependency of [WORKSPACE] or other [DEPENDENCY] package.
    pub const DEPENDENCY: RustPackageOrigin = RustPackageOrigin::new("dependency");
    /// The package comes from the standard library.
    pub const STDLIB: RustPackageOrigin = RustPackageOrigin::new("stdlib");
    /// External dependency of [STDLIB] or other [STDLIB_DEPENDENCY] package.
    pub const STDLIB_DEPENDENCY: RustPackageOrigin = RustPackageOrigin::new("stdlib-dependency");
    /// The package is a part of our workspace.
    pub const WORKSPACE: RustPackageOrigin = RustPackageOrigin::new("workspace");

    pub const fn new(tag: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(tag))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_package_origin() {
        assert_json_snapshot!(RustPackageOrigin::STDLIB, @r#""stdlib""#);
        assert_json_snapshot!(RustPackageOrigin::WORKSPACE, @r#""workspace""#);
        assert_json_snapshot!(RustPackageOrigin::DEPENDENCY, @r#""dependency""#);
        assert_json_snapshot!(RustPackageOrigin::STDLIB_DEPENDENCY, @r#""stdlib-dependency""#);
    }
}
