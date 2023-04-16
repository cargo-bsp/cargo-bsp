use std::path::PathBuf;

use log::error;

use crate::bsp_types::requests::BuildClientCapabilities;
use crate::project_model::ProjectManifest;

#[derive(Debug, Clone)]
pub struct Config {
    pub workspace_manifest: ProjectManifest,
    pub caps: BuildClientCapabilities,
    root_path: PathBuf,
}

impl Config {
    pub fn new(root_path: PathBuf, caps: BuildClientCapabilities) -> Self {
        let mut this = Config {
            workspace_manifest: ProjectManifest::default(),
            caps,
            root_path,
        };
        this.update_project_manifest();
        this
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn update_project_manifest(&mut self) {
        match ProjectManifest::discover(&self.root_path) {
            Ok(workspace_manifest) => {
                self.workspace_manifest = workspace_manifest;
            }
            Err(e) => {
                error!("No Cargo.toml found: {}", e);
                todo!("Add Logging to client and change server state to waiting for reload");
            }
        }
    }
}
