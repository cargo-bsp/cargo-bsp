use crate::requests::Request;
use crate::{BuildTargetIdentifier, Uri};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug)]
pub enum RustToolchain {}

impl Request for RustToolchain {
    type Params = RustToolchainParams;
    type Result = RustToolchainResult;
    const METHOD: &'static str = "buildTarget/rustToolchain";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainParams {
    /** A sequence of build targets for toolchain resolution. */
    pub targets: Vec<BuildTargetIdentifier>, // targety mogą mieć toolchainy różnego strumienia - stable - nigghtly itp
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainResult {
    /** A sequence of Rust toolchains. */
    pub toolchains: BTreeSet<RustToolchainItem>, // toolchain  dostępny systemowo, z którego korzysta cargo
}

#[derive(Serialize, Deserialize, Default, PartialOrd, PartialEq, Ord, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainItem {
    /** Additional information about Rust toolchain. */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_std_lib: Option<RustcInfo>,
    /** Path to Cargo executable. */
    pub cargo_bin_path: Uri,
    /** Location of the source code of procedural macros in the Rust toolchain. */
    pub proc_macro_srv_path: Uri,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialOrd, PartialEq, Ord, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RustcInfo {
    /** Root directory where the Rust compiler looks for standard libraries and other
    essential components when building Rust projects. */
    pub sysroot_path: Uri,
    /** Source code for the Rust standard library. */
    pub src_sysroot_path: Uri,
    /** `rustc` SemVer (Semantic Versioning) version. */
    pub version: String,
    /** Target architecture and operating system of the Rust compiler. */
    pub host: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::test_deserialization;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_toolchain_method() {
        assert_eq!(RustToolchain::METHOD, "buildTarget/rustToolchain");
    }

    #[test]
    fn rust_toolchain_params() {
        test_deserialization(
            r#"{"targets":[{"uri":""}]}"#,
            &RustToolchainParams {
                targets: vec![BuildTargetIdentifier::default()],
            },
        );
        test_deserialization(r#"{"targets":[]}"#, &RustToolchainParams::default());
    }

    #[test]
    fn rust_toolchain_result() {
        let result = RustToolchainResult {
            toolchains: BTreeSet::from([RustToolchainItem::default()]),
        };
        assert_json_snapshot!(result, @r###"
        {
          "toolchains": [
            {
              "cargoBinPath": "",
              "procMacroSrvPath": ""
            }
          ]
        }
        "###);

        assert_json_snapshot!(RustToolchainResult::default(), @r###"
        {
          "toolchains": []
        }
        "###);
    }

    #[test]
    fn rust_toolchain() {
        let rust_toolchain = RustToolchainItem {
            rust_std_lib: Some(RustcInfo::default()),
            cargo_bin_path: "test_cargo_bin_path".to_string(),
            proc_macro_srv_path: "test_proc_macro_srv_path".to_string(),
        };

        assert_json_snapshot!(rust_toolchain, @r###"
        {
          "rustStdLib": {
            "sysrootPath": "",
            "srcSysrootPath": "",
            "version": "",
            "host": ""
          },
          "cargoBinPath": "test_cargo_bin_path",
          "procMacroSrvPath": "test_proc_macro_srv_path"
        }
        "###);

        assert_json_snapshot!(RustToolchainItem::default(), @r###"
        {
          "cargoBinPath": "",
          "procMacroSrvPath": ""
        }
        "###);
    }

    #[test]
    fn rustc_info() {
        let rustc_info = RustcInfo {
            sysroot_path: "test_sysroot".to_string(),
            src_sysroot_path: "test_src_sysroot".to_string(),
            version: "test_version".to_string(),
            host: "test_host".to_string(),
        };
        assert_json_snapshot!(rustc_info, @r###"
        {
          "sysrootPath": "test_sysroot",
          "srcSysrootPath": "test_src_sysroot",
          "version": "test_version",
          "host": "test_host"
        }
        "###);

        assert_json_snapshot!(RustcInfo::default(), @r###"
        {
          "sysrootPath": "",
          "srcSysrootPath": "",
          "version": "",
          "host": ""
        }
        "###);
    }
}
