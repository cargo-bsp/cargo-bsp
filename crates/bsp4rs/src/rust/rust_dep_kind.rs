use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RustDepKind(pub std::borrow::Cow<'static, str>);

impl RustDepKind {
    /// For [build-dependencies].
    pub const BUILD: RustDepKind = RustDepKind::new("build");
    /// For [dev-dependencies].
    pub const DEV: RustDepKind = RustDepKind::new("dev");
    /// For [dependencies].
    pub const NORMAL: RustDepKind = RustDepKind::new("normal");
    /// For old Cargo versions prior to `1.41.0`.
    pub const UNCLASSIFIED: RustDepKind = RustDepKind::new("unclassified");

    pub const fn new(tag: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(tag))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_dep_kind() {
        assert_json_snapshot!(RustDepKind::UNCLASSIFIED, @r#""unclassified""#);
        assert_json_snapshot!(RustDepKind::NORMAL, @r#""normal""#);
        assert_json_snapshot!(RustDepKind::DEV, @r#""dev""#);
        assert_json_snapshot!(RustDepKind::BUILD, @r#""build""#);
    }
}
