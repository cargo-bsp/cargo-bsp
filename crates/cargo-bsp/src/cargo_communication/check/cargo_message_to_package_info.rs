//! Maps information from Cargo Messages produces by `cargo check` command to
//! RustPackage information.

use crate::utils::uri::file_uri;
use bsp_types::Uri;
use cargo_metadata::{Artifact, BuildScript, Package};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

const DYNAMIC_LIBRARY_EXTENSIONS: [&str; 3] = ["dll", "so", "dylib"];
const PROC_MACRO: &str = "proc-macro";

pub(super) fn map_cfg_options(script: Option<&BuildScript>) -> HashMap<String, Vec<String>> {
    script.map_or(HashMap::new(), |s| {
        let mut cfg_options: HashMap<String, Vec<String>> = HashMap::new();

        s.cfgs.iter().for_each(|cfg| {
            let mut parts = cfg.splitn(2, '=');
            let key = parts.next();
            let value = parts.next().map(|v| v.trim_matches('"').to_string());

            if let Some(k) = key {
                if let Some(v) = value {
                    if let Entry::Vacant(e) = cfg_options.entry(k.to_string()) {
                        e.insert(vec![v]);
                    } else {
                        cfg_options.get_mut(k).unwrap().push(v);
                    }
                } else {
                    cfg_options.insert(k.to_string(), vec![]);
                }
            }
        });

        cfg_options
    })
}

pub(super) fn map_env(script: Option<&BuildScript>, package: &Package) -> HashMap<String, String> {
    let version = package.version.clone();
    let mut env: HashMap<String, String> = HashMap::from([
        (
            "CARGO_MANIFEST_DIR",
            package
                .manifest_path
                .parent()
                .map(|p| p.to_string())
                .unwrap_or_default(),
        ),
        ("CARGO", "cargo".to_string()),
        ("CARGO_PKG_VERSION", package.version.to_string()),
        ("CARGO_PKG_VERSION_MAJOR", version.major.to_string()),
        ("CARGO_PKG_VERSION_MINOR", version.minor.to_string()),
        ("CARGO_PKG_VERSION_PATCH", version.patch.to_string()),
        ("CARGO_PKG_VERSION_PRE", version.pre.to_string()),
        ("CARGO_PKG_AUTHORS", package.authors.join(";")),
        ("CARGO_PKG_NAME", package.name.clone()),
        (
            "CARGO_PKG_DESCRIPTION",
            package.description.clone().unwrap_or_default(),
        ),
        (
            "CARGO_PKG_REPOSITORY",
            package.repository.clone().unwrap_or_default(),
        ),
        (
            "CARGO_PKG_LICENSE",
            package.license.clone().unwrap_or_default(),
        ),
        (
            "CARGO_PKG_LICENSE_FILE",
            package.license_file.clone().unwrap_or_default().to_string(),
        ),
        ("CARGO_CRATE_NAME", package.name.replace('-', "_")),
    ])
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect();
    if let Some(s) = script {
        for (k, v) in &s.env {
            env.insert(k.clone(), v.clone());
        }
    }
    env
}

pub(super) fn map_out_dir_url(script: Option<&BuildScript>) -> Option<String> {
    script.map(|s| file_uri(s.out_dir.to_string()))
}

pub(super) fn map_proc_macro_artifact(artifacts: &[Artifact]) -> Option<Uri> {
    artifacts
        .iter()
        .filter(|a| {
            a.target.kind.contains(&PROC_MACRO.to_string())
                && a.target.crate_types.contains(&PROC_MACRO.to_string())
        })
        .flat_map(|a| a.filenames.clone())
        .find(|f| {
            DYNAMIC_LIBRARY_EXTENSIONS
                .iter()
                .any(|&e| f.extension().map_or(false, |ex| ex == e))
        })
        .map(|f| f.to_string())
}
