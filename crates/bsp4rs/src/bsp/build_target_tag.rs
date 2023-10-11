use serde::{Deserialize, Serialize};

/// A list of predefined tags that can be used to categorize build targets.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildTargetTag(pub std::borrow::Cow<'static, str>);

impl BuildTargetTag {
    /// Target contains source code for producing any kind of application, may have
    /// but does not require the `canRun` capability.
    pub const APPLICATION: BuildTargetTag = BuildTargetTag::new("application");
    /// Target contains source code to measure performance of a program, may have
    /// but does not require the `canRun` build target capability.
    pub const BENCHMARK: BuildTargetTag = BuildTargetTag::new("benchmark");
    /// Target contains source code for integration testing purposes, may have
    /// but does not require the `canTest` capability.
    /// The difference between "test" and "integration-test" is that
    /// integration tests traditionally run slower compared to normal tests
    /// and require more computing resources to execute.
    pub const INTEGRATION_TEST: BuildTargetTag = BuildTargetTag::new("integration-test");
    /// Target contains re-usable functionality for downstream targets. May have any
    /// combination of capabilities.
    pub const LIBRARY: BuildTargetTag = BuildTargetTag::new("library");
    /// Actions on the target such as build and test should only be invoked manually
    /// and explicitly. For example, triggering a build on all targets in the workspace
    /// should by default not include this target.
    /// The original motivation to add the "manual" tag comes from a similar functionality
    /// that exists in Bazel, where targets with this tag have to be specified explicitly
    /// on the command line.
    pub const MANUAL: BuildTargetTag = BuildTargetTag::new("manual");
    /// Target should be ignored by IDEs.
    pub const NO_IDE: BuildTargetTag = BuildTargetTag::new("no-ide");
    /// Target contains source code for testing purposes, may have but does not
    /// require the `canTest` capability.
    pub const TEST: BuildTargetTag = BuildTargetTag::new("test");

    pub const fn new(tag: &'static str) -> Self {
        Self(std::borrow::Cow::Borrowed(tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_deserialization;
    use insta::assert_json_snapshot;

    #[test]
    fn build_target_tag() {
        assert_json_snapshot!(BuildTargetTag::LIBRARY, @r#""library""#);
        assert_json_snapshot!(BuildTargetTag::APPLICATION, @r#""application""#);
        assert_json_snapshot!(BuildTargetTag::TEST, @r#""test""#);
        assert_json_snapshot!(BuildTargetTag::INTEGRATION_TEST, @r#""integration-test""#);
        assert_json_snapshot!(BuildTargetTag::BENCHMARK, @r#""benchmark""#);
        assert_json_snapshot!(BuildTargetTag::NO_IDE, @r#""no-ide""#);
        assert_json_snapshot!(BuildTargetTag::MANUAL, @r#""manual""#);
        assert_json_snapshot!(BuildTargetTag::new("other"), @r#""other""#);

        test_deserialization(r#""library""#, &BuildTargetTag::LIBRARY);
        test_deserialization(r#""application""#, &BuildTargetTag::APPLICATION);
        test_deserialization(r#""test""#, &BuildTargetTag::TEST);
        test_deserialization(r#""integration-test""#, &BuildTargetTag::INTEGRATION_TEST);
        test_deserialization(r#""benchmark""#, &BuildTargetTag::BENCHMARK);
        test_deserialization(r#""no-ide""#, &BuildTargetTag::NO_IDE);
        test_deserialization(r#""manual""#, &BuildTargetTag::MANUAL);
        test_deserialization(r#""other""#, &BuildTargetTag::new("other"));
    }
}
