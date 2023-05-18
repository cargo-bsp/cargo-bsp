use crate::requests::Request;
use crate::BuildTargetIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RustToolchainReq {}

impl Request for RustToolchainReq {
    type Params = RustToolchainParams;
    type Result = RustToolchainResult;
    const METHOD: &'static str = "buildTarget/rustToolchain";
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainParams {
    pub targets: Vec<BuildTargetIdentifier>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchainResult {
    pub toolchains: Vec<RustToolchain>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustToolchain {
    pub rustc: RustcInfo,
    pub cargo_bin_path: String,
    pub proc_macro_srv_path: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RustcInfo {
    pub sysroot: String,
    pub src_sysroot: String,
    pub version: String,
    pub host: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use insta::assert_json_snapshot;

    #[test]
    fn rust_toolchain_method() {
        assert_eq!(RustToolchainReq::METHOD, "buildTarget/rustToolchain");
    }

    #[test]
    fn rust_toolchain_params() {
        let params = RustToolchainParams {
            targets: vec![BuildTargetIdentifier::default()],
        };
        assert_json_snapshot!(params, @r###"
        {
          "targets": [
            {
              "uri": ""
            }
          ]
        }
        "###);

        assert_json_snapshot!(RustToolchainParams::default(), @r###"
        {
          "targets": []
        }
        "###);
    }

    #[test]
    fn rust_toolchain_result() {
        let result = RustToolchainResult {
            toolchains: vec![RustToolchain::default()],
        };
        assert_json_snapshot!(result, @r###"
        {
          "toolchains": [
            {
              "rustc": {
                "sysroot": "",
                "srcSysroot": "",
                "version": "",
                "host": ""
              },
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
        let rust_toolchain = RustToolchain {
            rustc: RustcInfo::default(),
            cargo_bin_path: "test_cargo_bin_path".to_string(),
            proc_macro_srv_path: "test_proc_macro_srv_path".to_string(),
        };

        assert_json_snapshot!(rust_toolchain, @r###"
        {
          "rustc": {
            "sysroot": "",
            "srcSysroot": "",
            "version": "",
            "host": ""
          },
          "cargoBinPath": "test_cargo_bin_path",
          "procMacroSrvPath": "test_proc_macro_srv_path"
        }
        "###);

        assert_json_snapshot!(RustToolchain::default(), @r###"
        {
          "rustc": {
            "sysroot": "",
            "srcSysroot": "",
            "version": "",
            "host": ""
          },
          "cargoBinPath": "",
          "procMacroSrvPath": ""
        }
        "###);
    }

    #[test]
    fn rustc_info() {
        let rustc_info = RustcInfo {
            sysroot: "test_sysroot".to_string(),
            src_sysroot: "test_src_sysroot".to_string(),
            version: "test_version".to_string(),
            host: "test_host".to_string(),
        };
        assert_json_snapshot!(rustc_info, @r###"
        {
          "sysroot": "test_sysroot",
          "srcSysroot": "test_src_sysroot",
          "version": "test_version",
          "host": "test_host"
        }
        "###);

        assert_json_snapshot!(RustcInfo::default(), @r###"
        {
          "sysroot": "",
          "srcSysroot": "",
          "version": "",
          "host": ""
        }
        "###);
    }
}
