//! This file is a part of implementation to handle the BSP Rust extension.
//! Functions in this file are partially responsible
//! for preparing the data for RustWorkspaceRequest response.

use crate::project_model::rust_extension::metadata_edition_to_rust_extension_edition;
use bsp_types::extensions::{RustCrateType, RustTarget, RustTargetKind};

fn metadata_kind_to_rust_extension_kind(metadata_kind: &str) -> RustTargetKind {
    match metadata_kind {
        "lib" => RustTargetKind::Lib,
        "bin" => RustTargetKind::Bin,
        "test" => RustTargetKind::Test,
        "example" => RustTargetKind::Example,
        "bench" => RustTargetKind::Bench,
        "custom-build" => RustTargetKind::CustomBuild,
        _ => RustTargetKind::Unknown,
    }
}

fn metadata_crate_types_to_rust_extension_crate_types(
    metadata_crate_types: Vec<String>,
) -> Vec<RustCrateType> {
    metadata_crate_types
        .iter()
        .map(|mct| match mct.as_str() {
            "bin" => RustCrateType::Bin,
            "lib" => RustCrateType::Lib,
            "rlib" => RustCrateType::Rlib,
            "dylib" => RustCrateType::Dylib,
            "cdylib" => RustCrateType::Cdylib,
            "staticlib" => RustCrateType::Staticlib,
            "proc-macro" => RustCrateType::ProcMacro,
            _ => RustCrateType::Other,
        })
        .collect()
}

pub(crate) fn metadata_targets_to_rust_extension_targets(
    mut metadata_targets: Vec<cargo_metadata::Target>,
) -> Vec<RustTarget> {
    metadata_targets
        .iter_mut()
        .map(|mt| {
            RustTarget {
                name: mt.name.clone(),
                crate_root_url: "TODO".into(),   //TODO
                package_root_url: "TODO".into(), //TODO
                kind: metadata_kind_to_rust_extension_kind(mt.kind.get(0).unwrap().as_str()), // Cargo metadata target always has at least one kind.
                crate_types: metadata_crate_types_to_rust_extension_crate_types(
                    mt.crate_types.clone(),
                ),
                required_features: mt.required_features.clone(),
                doctest: mt.doctest,
                edition: metadata_edition_to_rust_extension_edition(mt.edition),
            }
        })
        .collect()
}
