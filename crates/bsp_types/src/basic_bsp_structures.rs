use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::extensions::CargoBuildTarget;

/** A resource identifier that is a valid URI according
to rfc3986: https://tools.ietf.org/html/rfc3986 */
pub type URI = String;

pub const RUST_ID: &str = "rust";

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherData {
    pub data_kind: String,
    pub data: serde_json::Value,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TextDocumentIdentifier {
    pub uri: URI,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildTarget {
    /** The target’s unique identifier */
    pub id: BuildTargetIdentifier,

    /** A human readable name for this target.
    May be presented in the user interface.
    Should be unique if possible.
    The id.uri is used if None. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /** The directory where this target belongs to. Multiple build targets are allowed to map
    to the same base directory, and a build target is not required to have a base directory.
    A base directory does not determine the sources of a target, see buildTarget/sources. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_directory: Option<URI>,

    /** Free-form string tags to categorize or label this build target.
    For example, can be used by the client to:
    * customize how the target should be translated into the client's project model.
    * group together different but related targets in the user interface.
    * display icons or colors in the user interface.

    Pre-defined tags are listed in `build_target_tag` but clients and servers
    are free to define new tags for custom purposes. */
    pub tags: Vec<BuildTargetTag>,

    /** The capabilities of this build target. */
    pub capabilities: BuildTargetCapabilities,

    /** The set of languages that this target contains.
    The ID string for each language is defined in the LSP. */
    pub language_ids: Vec<String>,

    /** The direct upstream build target dependencies of this build target */
    pub dependencies: Vec<BuildTargetIdentifier>,

    /** Language-specific metadata about this target.
    See ScalaBuildTarget as an example. */
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub data: Option<BuildTargetData>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "dataKind", content = "data")]
pub enum NamedBuildTargetData {
    Cargo(CargoBuildTarget),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BuildTargetData {
    Named(NamedBuildTargetData),
    Other(OtherData),
}

impl BuildTargetData {
    pub fn cargo(data: CargoBuildTarget) -> Self {
        BuildTargetData::Named(NamedBuildTargetData::Cargo(data))
    }
}

/** A unique identifier for a target, can use any URI-compatible encoding as long as it is unique
within the workspace. Clients should not infer metadata out of the URI structure such as the path
or query parameters, use BuildTarget instead.*/
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone, Hash)]
pub struct BuildTargetIdentifier {
    /** The target’s Uri */
    pub uri: URI,
}

/** A list of predefined tags that can be used to categorize build targets. */
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BuildTargetTag(pub std::borrow::Cow<'static, str>);
impl BuildTargetTag {
    /** Target contains source code for producing any kind of application, may have
    but does not require the `canRun` capability. */
    pub const APPLICATION: BuildTargetTag = BuildTargetTag::new("application");
    /** Target contains source code to measure performance of a program, may have
    but does not require the `canRun` build target capability. */
    pub const BENCHMARK: BuildTargetTag = BuildTargetTag::new("benchmark");
    /** Target contains source code for integration testing purposes, may have
    but does not require the `canTest` capability.
    The difference between "test" and "integration-test" is that
    integration tests traditionally run slower compared to normal tests
    and require more computing resources to execute. */
    pub const INTEGRATION_TEST: BuildTargetTag = BuildTargetTag::new("integration-test");
    /** Target contains re-usable functionality for downstream targets. May have any
    combination of capabilities. */
    pub const LIBRARY: BuildTargetTag = BuildTargetTag::new("library");
    /** Actions on the target such as build and test should only be invoked manually
    and explicitly. For example, triggering a build on all targets in the workspace
    should by default not include this target.
    The original motivation to add the "manual" tag comes from a similar functionality
    that exists in Bazel, where targets with this tag have to be specified explicitly
    on the command line.
     */
    pub const MANUAL: BuildTargetTag = BuildTargetTag::new("manual");
    /** Target should be ignored by IDEs. */
    pub const NO_IDE: BuildTargetTag = BuildTargetTag::new("no-ide");
    /** Target contains source code for testing purposes, may have but does not
    require the `canTest` capability. */
    pub const TEST: BuildTargetTag = BuildTargetTag::new("test");

    pub const fn new(tag: &'static str) -> Self {
        BuildTargetTag(std::borrow::Cow::Borrowed(tag))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuildTargetCapabilities {
    /** This target can be compiled by the BSP server. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_compile: Option<bool>,
    /** This target can be tested by the BSP server. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_test: Option<bool>,
    /** This target can be run by the BSP server. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_run: Option<bool>,
    /** This target can be debugged by the BSP server. */
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub can_debug: Option<bool>,
}

/** Included in notifications of tasks or requests to signal the completion state. */
#[derive(Debug, PartialEq, Serialize_repr, Deserialize_repr, Default, Clone)]
#[repr(u8)]
pub enum StatusCode {
    /** Execution was successful. */
    Ok = 1,
    /** Execution failed. */
    #[default]
    Error = 2,
    /** Execution was cancelled. */
    Cancelled = 3,
}

#[cfg(test)]
mod tests {
    use crate::tests::test_deserialization;
    use insta::assert_json_snapshot;

    use super::*;

    #[test]
    fn text_document_identifier() {
        let test_data = TextDocumentIdentifier {
            uri: "test_uri".to_string(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "uri": "test_uri"
        }
        "#
        );
        assert_json_snapshot!(TextDocumentIdentifier::default(),
            @r#"
        {
          "uri": ""
        }
        "#
        );
    }

    #[test]
    fn build_target() {
        let test_data = BuildTarget {
            id: BuildTargetIdentifier::default(),
            display_name: Some("test_displayName".to_string()),
            base_directory: Some("test_baseDirectory".to_string()),
            tags: vec![BuildTargetTag::TEST],
            capabilities: BuildTargetCapabilities::default(),
            language_ids: vec!["test_languageId".to_string()],
            dependencies: vec![BuildTargetIdentifier::default()],
            data: Some(BuildTargetData::cargo(CargoBuildTarget::default())),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "id": {
            "uri": ""
          },
          "displayName": "test_displayName",
          "baseDirectory": "test_baseDirectory",
          "tags": [
            "test"
          ],
          "capabilities": {},
          "languageIds": [
            "test_languageId"
          ],
          "dependencies": [
            {
              "uri": ""
            }
          ],
          "dataKind": "cargo",
          "data": {
            "edition": "",
            "requiredFeatures": []
          }
        }
        "#
        );
        assert_json_snapshot!(BuildTarget::default(),
            @r#"
        {
          "id": {
            "uri": ""
          },
          "tags": [],
          "capabilities": {},
          "languageIds": [],
          "dependencies": []
        }
        "#
        );
    }

    #[test]
    fn cargo_build_target_data() {
        assert_json_snapshot!(BuildTargetData::cargo(CargoBuildTarget::default()),
            @r#"
        {
          "dataKind": "cargo",
          "data": {
            "edition": "",
            "requiredFeatures": []
          }
        }
        "#
        );
    }

    #[test]
    fn build_target_identifier() {
        let test_data = BuildTargetIdentifier {
            uri: "test_uri".to_string(),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "uri": "test_uri"
        }
        "#
        );
        assert_json_snapshot!(BuildTargetIdentifier::default(),
            @r#"
        {
          "uri": ""
        }
        "#
        );
    }

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

    #[test]
    fn build_target_capabilities() {
        let test_data = BuildTargetCapabilities {
            can_compile: Some(true),
            can_test: Some(true),
            can_run: Some(true),
            can_debug: Some(true),
        };

        assert_json_snapshot!(test_data,
            @r#"
        {
          "canCompile": true,
          "canTest": true,
          "canRun": true,
          "canDebug": true
        }
        "#
        );
        assert_json_snapshot!(BuildTargetCapabilities::default(),
            @r#"
        {}
        "#
        );
    }

    #[test]
    fn status_code() {
        assert_json_snapshot!(StatusCode::Ok, @"1");
        assert_json_snapshot!(StatusCode::Error, @"2");
        assert_json_snapshot!(StatusCode::Cancelled, @"3");
    }
}
