// copy from rust-analyzer

use std::path::PathBuf;

use crate::bsp_types::BuildClientCapabilities;
use crate::project_model::ProjectManifest;

#[derive(Debug, Clone)]
pub struct Config {
    pub discovered_projects: Vec<ProjectManifest>,
    pub caps: BuildClientCapabilities,
    root_path: PathBuf,
}

impl Config {
    pub fn new(root_path: PathBuf, caps: BuildClientCapabilities) -> Self {
        Config {
            discovered_projects: vec![],
            caps,
            root_path,
        }
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }
}

impl Config {
    pub fn linked_projects(&self) -> Vec<ProjectManifest> {
        self.discovered_projects
            .iter()
            .cloned()
            .map(ProjectManifest::from)
            .collect()
    }
}
