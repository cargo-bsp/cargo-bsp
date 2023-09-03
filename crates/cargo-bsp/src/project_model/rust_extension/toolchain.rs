//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are responsible for preparing the data for RustToolchainRequest response.

use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::{RustToolchainItem, RustcInfo};
use bsp_types::BuildTargetIdentifier;
use log::warn;
use rustc_version::{version, version_meta};
use std::collections::BTreeSet;
use std::ops::Add;

fn get_sysroot() -> Option<String> {
    let output = std::process::Command::new(toolchain::rustc())
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;

    Some(stdout.trim().to_string())
}

fn establish_rustc_info_for_target(_build_target_id: &BuildTargetIdentifier) -> RustcInfo {
    let sysroot_path = get_sysroot()
        .or_else(|| {
            warn!("Failed to obtain rustc sysroot path. Using 'Unknown' instead.");
            Some("Unknown".to_string())
        })
        .unwrap();

    let version = if let Ok(v) = version() {
        v.to_string()
    } else {
        warn!("Failed to obtain rustc version. Using 'Unknown' instead.");
        "Unknown".to_string()
    };

    let host = if let Ok(v) = version_meta() {
        v.host
    } else {
        warn!("Failed to obtain rustc host. Using 'Unknown' instead.");
        "Unknown".to_string()
    };

    RustcInfo {
        src_sysroot_path: sysroot_path.clone().add("/lib/rustlib/src/rust"),
        sysroot_path,
        version,
        host,
    }
}

// Currently responds with toolchain used in a root of the directory, as
// [intellij-rust fork](https://github.com/ZPP-This-is-fine/intellij-rust) does not
// support toolchain resolution for specific targets.
// In the future it should respond with the list of toolchains which are used within the project.
// This can be done by calling the `rustc --version --verbose` in the directory where each of the targets is located.
pub fn get_rust_toolchains(
    _workspace: &ProjectWorkspace,
    build_target_ids: Vec<BuildTargetIdentifier>,
) -> BTreeSet<RustToolchainItem> {
    build_target_ids
        .iter()
        .map(|id| {
            let rustc_info = establish_rustc_info_for_target(id);
            let cargo_bin_path = toolchain::cargo().to_string_lossy().to_string();
            RustToolchainItem {
                cargo_bin_path,
                proc_macro_srv_path: rustc_info
                    .sysroot_path
                    .clone()
                    .add("/libexec/rust-analyzer-proc-macro-srv"),
                rust_std_lib: Some(rustc_info),
            }
        })
        .collect()
}
