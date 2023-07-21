use crate::project_model::workspace::ProjectWorkspace;
use bsp_types::rust_extension::{RustToolchainsItem, RustcInfo};
use bsp_types::BuildTargetIdentifier;
use std::ops::Add;

pub fn establish_rustc_info_for_target(_build_target_id: &BuildTargetIdentifier) -> RustcInfo {
    let sysroot_path: String = "".into(); //todo rustc -print sysroot
    RustcInfo {
        src_sysroot_path: sysroot_path.clone().add("/lib/rustlib/src/rust"),
        sysroot_path,
        version: "1.0".to_string(), //todo rustc --version --version + regex
        host: "".into(),            //todo rustc --version --version + regex,
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
            let cargo_bin_path = "".to_string(); //todo
            RustToolchainsItem {
                cargo_bin_path,
                proc_macro_srv_path: "".to_string(), //todo  ///home/tudny/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/libexec/rust-analyzer-proc-macro-srv, scieżka do binraki rozwijającej makra proceduralne
                rust_std_lib: Some(rustc_info),
            }
        })
        .collect()
}
