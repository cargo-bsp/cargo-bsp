// copy from rust-analyzer

use std::path::PathBuf;

use crate::bsp_types::BuildClientCapabilities;
use crate::project_model::ProjectManifest;

#[derive(Debug, Clone)]
pub struct Config {
    pub workspace_manifest: ProjectManifest,
    pub caps: BuildClientCapabilities,
    root_path: PathBuf,
}

impl Config {
    pub fn new(root_path: PathBuf, caps: BuildClientCapabilities, ) -> Self {
        Config {
            workspace_manifest: ProjectManifest::discover_all(&root_path),
            caps,
            root_path,
        }
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }
}