use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::extensions::{RustToolchainsItem, RustcInfo};
use bsp_types::BuildTargetIdentifier;
use log::warn;
use rustc_version::{version, version_meta};
use std::ops::Add;

fn get_sysroot() -> Option<String> {
    let output = std::process::Command::new(toolchain::rustc())
        .arg("--print")
        .arg("sysroot")
        .output()
        .ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;

    Some(stdout.trim().to_string())
}

fn establish_rustc_info_for_target(_build_target_id: &BuildTargetIdentifier) -> RustcInfo {
    let sysroot_path: String = get_sysroot()
        .or_else(|| {
            warn!("Failed to obtain rustc sysroot path. Using 'Unknown' instead.");
            Some("Unknown".to_string())
        })
        .unwrap();

    let rustc_version;
    if let Ok(v) = version() {
        rustc_version = v.to_string();
    } else {
        warn!("Failed to obtain rustc version. Using 'Unknown' instead.");
        rustc_version = "Unknown".to_string();
    }

    let host;
    if let Ok(v) = version_meta() {
        host = v.host;
    } else {
        warn!("Failed to obtain rustc host. Using 'Unknown' instead.");
        host = "Unknown".to_string();
    }

    RustcInfo {
        src_sysroot_path: sysroot_path.clone().add("/lib/rustlib/src/rust"),
        sysroot_path,
        version: rustc_version,
        host,
    }
}

pub fn get_rust_toolchain_items(
    _workspace: &ProjectWorkspace,
    build_target_ids: Vec<BuildTargetIdentifier>,
) -> Vec<RustToolchainsItem> {
    build_target_ids
        .iter()
        .map(|id| {
            let rustc_info = establish_rustc_info_for_target(id);
            let cargo_bin_path = toolchain::cargo().to_string_lossy().to_string(); //todo
            RustToolchainsItem {
                cargo_bin_path,
                proc_macro_srv_path: rustc_info
                    .clone()
                    .sysroot_path
                    .add("/libexec/rust-analyzer-proc-macro-srv"),
                rust_std_lib: Some(rustc_info),
            }
        })
        .collect()
}
