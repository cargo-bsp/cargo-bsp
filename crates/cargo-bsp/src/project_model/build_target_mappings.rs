//! Maps Cargo metadata target to the BSP build target.

use std::fmt::Display;
use std::rc::Rc;

use cargo_metadata::camino::Utf8PathBuf;
use log::warn;

use crate::project_model::metadata_edition_to_bsp_edition;
use bsp_types::basic_bsp_structures::*;

use crate::utils::uri::file_uri;

pub fn build_target_id_from_name_and_path<T: Display, R: Display>(
    name: T,
    path: R,
) -> BuildTargetIdentifier {
    BuildTargetIdentifier {
        uri: format!("targetId:/{}:{}", path, name),
    }
}

/// We assume that this function is called only with valid path which has a parent
pub fn parent_path(path: &Utf8PathBuf) -> Utf8PathBuf {
    let mut parent_directory = path.clone();
    parent_directory.pop();
    parent_directory
}

pub fn path_parent_directory_uri(path: &Utf8PathBuf) -> Uri {
    file_uri(parent_path(path))
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
            }
            "bin" => {
                tags.push(BuildTargetTag::Application);
            }
            "example" => {
                tags.push(BuildTargetTag::Application);
                capabilities.can_test = false;
            }
            "test" => {
                tags.push(BuildTargetTag::IntegrationTest);
                capabilities.can_run = false;
            }
            "bench" => {
                tags.push(BuildTargetTag::Benchmark);
                capabilities.can_run = false;
            }
            "custom-build" => {
                //TODO
                warn!("Found Custom-Build target, which is unsupported by BSP server yet.")
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

    let rust_specific_data = BuildTargetData::Cargo(CargoBuildTarget {
        edition: metadata_edition_to_bsp_edition(cargo_target.edition),
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

pub fn build_target_ids_from_cargo_targets(
    cargo_targets: &[Rc<cargo_metadata::Target>],
) -> Vec<BuildTargetIdentifier> {
    cargo_targets
        .iter()
        .map(|target| build_target_id_from_name_and_path(&target.name, &target.src_path))
        .collect()
}
