use std::path::PathBuf;
use std::sync::Arc;

use cargo_metadata::camino::Utf8PathBuf;
use log::warn;
use walkdir::WalkDir;

use bsp_types::requests::{SourceItem, SourceItemKind, SourcesItem};
use bsp_types::BuildTargetIdentifier;

use crate::project_model::target_details::{CargoTargetKind, TargetDetails};
use crate::project_model::workspace::ProjectWorkspace;
use crate::utils::uri::file_uri;

pub fn get_sources_item(
    workspace: &Arc<ProjectWorkspace>,
    id: BuildTargetIdentifier,
) -> Option<SourcesItem> {
    let target_details = workspace.get_target_details(&id).or_else(|| {
        warn!("Failed to get target details for: {:?}", id);
        None
    })?;
    let package_path = target_details.package_abs_path.clone();

    Some(SourcesItem {
        target: id,
        sources: list_target_sources(target_details),
        roots: vec![file_uri(package_path)],
    })
}

fn list_target_sources(target_details: TargetDetails) -> Vec<SourceItem> {
    fn list_source_files_in_path(path: Utf8PathBuf) -> Vec<SourceItem> {
        get_all_rs_files_in_dir(path.as_str())
            .into_iter()
            .map(create_source_item)
            .collect()
    }

    let package_path = target_details.package_abs_path.clone();

    let mut src_sources: Vec<SourceItem> = list_source_files_in_path(package_path.join("src"));

    match target_details.kind {
        CargoTargetKind::Lib | CargoTargetKind::Bin => {}
        _ => src_sources.append(&mut list_source_files_in_path(package_path.join("tests"))),
    }

    src_sources
}

fn get_all_rs_files_in_dir(dir: &str) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let f_name = e.file_name().to_string_lossy();
                if f_name.ends_with(".rs") {
                    Some(e.into_path())
                } else {
                    None
                }
            })
        })
        .collect()
}

fn create_source_item(source_path: PathBuf) -> SourceItem {
    let source_kind = if source_path.is_dir() {
        SourceItemKind::Directory
    } else {
        SourceItemKind::File
    };

    return SourceItem {
        uri: file_uri(source_path.to_str().unwrap()),
        kind: source_kind,
        generated: false,
    };
}
