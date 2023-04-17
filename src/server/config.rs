use std::env;
use std::path::PathBuf;

use log::error;
use url::Url;

use crate::bsp_types::requests::{BuildClientCapabilities, InitializeBuildParams};
use crate::project_model::project_manifest::ProjectManifest;
use crate::server::Result;

#[derive(Debug, Clone)]
pub struct Config {
    // we assume project has only one workspace, therefore one root Cargo.toml - ProjectManifest
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

    pub(crate) fn from_initialize_params(
        initialize_params: InitializeBuildParams,
    ) -> Result<Config> {
        let root_path = match Url::try_from(initialize_params.root_uri.as_str())
            .ok()
            .and_then(|it| it.to_file_path().ok())
        {
            Some(it) => it,
            None => env::current_dir()?,
        };

        Ok(Config::new(root_path, initialize_params.capabilities))
    }
}
