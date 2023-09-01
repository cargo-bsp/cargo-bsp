use crate::requests::Request;
use crate::{BuildTargetIdentifier, URI};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RustToolchainReq {}

impl Request for RustToolchainReq {
    type Params = RustToolchainParams;
    type Result = RustToolchainResult;
    const METHOD: &'static str = "buildTarget/rustToolchain";
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainParams {
    pub targets: Vec<BuildTargetIdentifier>, // targety mogą mieć toolchainy różnego strumienia - stable - nigghtly itp
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainResult {
    pub items: Vec<RustToolchainsItem>, // toolchain  dostępny systemowo, z którego korzysta cargo
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainsItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rust_std_lib: Option<RustcInfo>,
    pub cargo_bin_path: URI,
    pub proc_macro_srv_path: URI, // scieżka do binraki rozwijającej makra proceduralne
}
///home/tudny/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/libexec/rust-analyzer-proc-macro-srv

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustcInfo {
    pub sysroot_path: URI,
    pub src_sysroot_path: URI,
    ///home/tudny/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/libexec/rust-analyzer-proc-macro-srv
    pub version: String,
    pub host: String, //example: x86_64-unknown-linux-gnu rustc --version --verbose
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::test_deserialization;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_toolchain_method() {
        assert_eq!(RustToolchainReq::METHOD, "buildTarget/rustToolchain");
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
            items: vec![RustToolchainsItem::default()],
        };
        assert_json_snapshot!(result, @r#"
        {
          "items": [
            {
              "cargoBinPath": "",
              "procMacroSrvPath": ""
            }
          ]
        }
        "#);

        assert_json_snapshot!(RustToolchainResult::default(), @r#"
        {
          "items": []
        }
        "#);
    }

    #[test]
    fn rust_toolchain() {
        let rust_toolchain = RustToolchainsItem {
            rust_std_lib: Some(RustcInfo::default()),
            cargo_bin_path: "test_cargo_bin_path".into(),
            proc_macro_srv_path: "test_proc_macro_srv_path".into(),
        };

        assert_json_snapshot!(rust_toolchain, @r#"
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
        "#);

        assert_json_snapshot!(RustToolchainsItem::default(), @r#"
        {
          "cargoBinPath": "",
          "procMacroSrvPath": ""
        }
        "#);
    }

    #[test]
    fn rustc_info() {
        let rustc_info = RustcInfo {
            sysroot_path: "test_sysroot".into(),
            src_sysroot_path: "test_src_sysroot".into(),
            version: "test_version".to_string(),
            host: "test_host".to_string(),
        };
        assert_json_snapshot!(rustc_info, @r#"
        {
          "sysrootPath": "test_sysroot",
          "srcSysrootPath": "test_src_sysroot",
          "version": "test_version",
          "host": "test_host"
        }
        "#);

        assert_json_snapshot!(RustcInfo::default(), @r#"
        {
          "sysrootPath": "",
          "srcSysrootPath": "",
          "version": "",
          "host": ""
        }
        "#);
    }
}

// Q: Czy zakładamy, że jak nie ma not null, to jest optional?
// Q: all_targets w package?
// Q: RustcInfo: src_sysroot, host?
// Q: ProcMacroSrvPath, błąd? Co to? i czy jest target specific? (Vec[buildTargetIdentifier])
