use bsp_types::extensions::{RustCfgOptions, RustProcMacroArtifact};
use cargo_metadata::BuildScript;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

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

pub(super) fn map_env(script: Option<&BuildScript>, _version: String) -> HashMap<String, String> {
    let env: HashMap<String, String> = HashMap::new();
    if let Some(s) = script {
        s.env.iter().for_each(|(_key, _value)| {
            // let new_value = match key.as_str() {
            //     // todo get root path
            //     "CARGO_MANIFEST_DIR" => "a".to_string(),
            //     "CARGO" => "cargo",
            //     "CARGO_PKG_VERSION" => version,
            //     "CARGO_PKG_VERSION_MAJOR" => semver?.major?.toString().orEmpty(),
            //     "CARGO_PKG_VERSION_MINOR" => semver?.minor?.toString().orEmpty(),
            //     "CARGO_PKG_VERSION_PATCH" => semver?.patch?.toString().orEmpty(),
            //     "CARGO_PKG_VERSION_PRE" => semver?.preRelease.orEmpty(),
            //     "CARGO_PKG_AUTHORS" => authors.joinToString(separator = ";"),
            //     "CARGO_PKG_NAME" => name,
            //     "CARGO_PKG_DESCRIPTION" => description.orEmpty(),
            //     "CARGO_PKG_REPOSITORY" => repository.orEmpty(),
            //     "CARGO_PKG_LICENSE" => license.orEmpty(),
            //     "CARGO_PKG_LICENSE_FILE" => license_file.orEmpty(),
            //     "CARGO_CRATE_NAME" => name.replace('-', '_'),
            //     _ => String::default(),
            // };
        })
    }
    env
}

pub(super) fn map_out_dir_url(_script: Option<&BuildScript>) -> Option<String> {
    None
}

pub(super) fn map_proc_macro_artifact(
    _script: Option<&BuildScript>,
) -> Option<RustProcMacroArtifact> {
    None
}
