//! Maps information from Cargo Messages produces by `cargo check` command to
//! RustPackage information.

use crate::utils::uri::file_uri;
use bsp_types::extensions::RustCfgOptions;
use bsp_types::Uri;
use cargo_metadata::{Artifact, BuildScript, Package};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

const DYNAMIC_LIBRARY_EXTENSIONS: [&str; 3] = ["dll", "so", "dylib"];
const PROC_MACRO: &str = "proc-macro";

#[derive(Default)]
struct SplitVersion {
    major: String,
    minor: String,
    pre_release: String,
    patch: String,
}

pub(super) fn map_cfg_options(script: Option<&BuildScript>) -> Option<RustCfgOptions> {
    script.map(|s| {
        let mut key_value_options: HashMap<String, Vec<String>> = HashMap::new();
        let mut name_options = Vec::new();

        s.cfgs.iter().for_each(|cfg| {
            let mut parts = cfg.splitn(2, '=');
            let key = parts.next();
            let value = parts.next().map(|v| v.trim_matches('"').to_string());

            if let Some(k) = key {
                if let Some(v) = value {
                    if let Entry::Vacant(e) = key_value_options.entry(k.to_string()) {
                        e.insert(vec![v]);
                    } else {
                        key_value_options.get_mut(k).unwrap().push(v);
                    }
                } else {
                    name_options.push(k.to_string());
                }
            }
        });

        RustCfgOptions {
            key_value_options,
            name_options,
        }
    })
}

fn split_version(version: String) -> SplitVersion {
    if let Some((major, rest)) = version.split_once('.') {
        if let Some((minor, rest)) = rest.split_once('.') {
            let (patch, pre_release) = rest
                .split_once('-')
                .map(|(s1, s2)| (s1.to_string(), s2.to_string()))
                .unwrap_or((rest.to_string(), String::default()));
            return SplitVersion {
                major: major.to_string(),
                minor: minor.to_string(),
                pre_release,
                patch,
            };
        }
    }
    SplitVersion::default()
}

pub(super) fn map_env(script: Option<&BuildScript>, package: &Package) -> HashMap<String, String> {
    let split_version = split_version(package.version.to_string());
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
        ("CARGO_PKG_VERSION_MAJOR", split_version.major.clone()),
        ("CARGO_PKG_VERSION_MINOR", split_version.minor.clone()),
        ("CARGO_PKG_VERSION_PATCH", split_version.patch.clone()),
        ("CARGO_PKG_VERSION_PRE", split_version.pre_release),
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
