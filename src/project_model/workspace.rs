use crate::bsp_types::BuildTarget;
use cargo_metadata::{CargoOpt, MetadataCommand, Package};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    pub _packages: Vec<Package>,
    pub _cargo_targets: Vec<cargo_metadata::Target>,
    build_targets: Vec<BuildTarget>,
}

impl ProjectWorkspace {
    pub fn new(packages: Vec<&Package>) -> ProjectWorkspace {
        let cargo_targets = ProjectWorkspace::cargo_targets(&packages);
        let targets = ProjectWorkspace::bsp_targets_from_metadata_packages(&packages);
        ProjectWorkspace {
            _packages: packages.into_iter().cloned().collect(),
            _cargo_targets: cargo_targets,
            build_targets: targets,
        }
    }

    fn cargo_targets(packages: &[&Package]) -> Vec<cargo_metadata::Target> {
        packages
            .iter()
            .flat_map(|package| package.targets.iter())
            .cloned()
            .collect()
    }

    // If we decide to keep the targets as a vector of cargo_targets, we can use _cargo_targets
    fn bsp_targets_from_metadata_packages(packages: &[&Package]) -> Vec<BuildTarget> {
        packages
            .iter()
            .flat_map(|package| package.targets.iter())
            .map(BuildTarget::from)
            .collect()
    }

    pub fn get_build_targets(&self) -> Vec<BuildTarget> {
        self.build_targets.clone()
    }
}

impl From<PathBuf> for ProjectWorkspace {
    fn from(project_manifest_path: PathBuf) -> Self {
        let metadata = MetadataCommand::new()
            .manifest_path(project_manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec()
            .unwrap();

        ProjectWorkspace::new(metadata.workspace_packages())
    }
}
