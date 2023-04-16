use crate::bsp_types::basic_bsp_structures::*;
use crate::project_model::package_dependencies::PackageDependency;
use log::{error, warn};

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

fn establish_dependencies(
    package_dependencies: &[PackageDependency],
) -> Vec<BuildTargetIdentifier> {
    let dependencies_manifest_paths = package_dependencies.iter().filter_map(|dep| {
        let manifest_path_str = dep.manifest_path.to_str();
        if manifest_path_str.is_none() {
            error!(
                "Failed extracting manifest path from dependency: {:?}",
                dep.manifest_path
            );
        }
        manifest_path_str
    });

    dependencies_manifest_paths
        .map(|path| BuildTargetIdentifier {
            uri: format!("file://{}", path),
        })
        .collect()
}

pub fn new_bsp_build_target(
    cargo_target: &cargo_metadata::Target,
    package_dependencies: &[PackageDependency],
) -> BuildTarget {
    let mut base_directory = cargo_target.src_path.clone();
    // we assume that cargo metadata returns valid path to file, which additionally has a parent
    base_directory.pop();

    let (tags, capabilities) = tags_and_capabilities_from_cargo_kind(cargo_target);

    let rust_specific_data = RustBuildTargetData::Rust(RustBuildTarget {
        edition: cargo_target.edition,
        required_features: cargo_target.required_features.clone(),
    });

    BuildTarget {
        id: BuildTargetIdentifier {
            uri: format!("{}:{}", cargo_target.src_path, cargo_target.name),
        },
        display_name: Some(cargo_target.name.clone()),
        base_directory: Some(format!("file://{}", base_directory)),
        tags,
        capabilities,
        language_ids: vec![RUST_ID.to_string()],
        dependencies: establish_dependencies(package_dependencies),
        data: Some(rust_specific_data),
    }
}
