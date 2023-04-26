use crate::bsp_types::basic_bsp_structures::*;
use crate::bsp_types::mappings::file_uri;
use cargo_metadata::camino::Utf8PathBuf;
use log::warn;
use std::fmt::Display;

pub fn build_target_id_from_name_and_path<T: Display, R: Display>(
    name: T,
    path: R,
) -> BuildTargetIdentifier {
    BuildTargetIdentifier {
        uri: format!("targetId:/{}:{}", path, name),
    }
}

/// Assumes calling only with valid path which additionally has a parent
pub fn path_parent_directory_uri(path: &Utf8PathBuf) -> Uri {
    let mut parent_directory = path.clone();
    parent_directory.pop();
    file_uri(parent_directory)
}

fn tags_and_capabilities_from_cargo_kind(
    cargo_target: &cargo_metadata::Target,
) -> (Vec<BuildTargetTag>, BuildTargetCapabilities) {
    let mut tags = vec![];
    let mut capabilities = BuildTargetCapabilities {
        can_compile: true,
        can_test: true,
        can_run: true,
        can_debug: true,
    };
    cargo_target
        .kind
        .iter()
        .for_each(|kind| match kind.as_str() {
            "lib" => {
                tags.push(BuildTargetTag::Library);
                capabilities.can_debug = false;
                capabilities.can_run = false;
                capabilities.can_test = false;
            }
            "bin" => {
                tags.push(BuildTargetTag::Application);
                capabilities.can_test = false;
            }
            "example" => {
                tags.push(BuildTargetTag::Application);
                capabilities.can_test = false;
            }
            "test" => {
                tags.push(BuildTargetTag::Test);
                capabilities.can_run = false;
            }
            "bench" => {
                tags.push(BuildTargetTag::Benchmark);
                capabilities.can_run = false;
            }
            "custom-build" => {
                todo!("Custom-build target is unsupported by BSP server yet.");
            }
            _ => {
                warn!("Unknown cargo target kind: {}", kind);
            }
        });
    (tags, capabilities)
}

pub fn bsp_build_target_from_cargo_target(
    cargo_target: &cargo_metadata::Target,
    target_dependencies: &[BuildTargetIdentifier],
) -> BuildTarget {
    let (tags, capabilities) = tags_and_capabilities_from_cargo_kind(cargo_target);

    let rust_specific_data = RustBuildTargetData::Rust(RustBuildTarget {
        edition: cargo_target.edition,
        required_features: cargo_target.required_features.clone(),
    });

    BuildTarget {
        id: build_target_id_from_name_and_path(&cargo_target.name, &cargo_target.src_path),
        display_name: Some(cargo_target.name.clone()),
        // We assume that cargo metadata always returns valid paths, which additionally have a parent
        base_directory: Some(path_parent_directory_uri(&cargo_target.src_path)),
        tags,
        capabilities,
        language_ids: vec![RUST_ID.to_string()],
        dependencies: Vec::from(target_dependencies),
        data: Some(rust_specific_data),
    }
}
