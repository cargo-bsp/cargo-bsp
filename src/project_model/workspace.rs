use std::path::PathBuf;
use cargo_metadata::{CargoOpt, MetadataCommand, Package};
use crate::bsp_types::BuildTarget;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    pub _packages: Vec<Package>,
    pub build_targets: Vec<BuildTarget>,
}

impl ProjectWorkspace {
    pub fn new(packages: Vec<&Package>) -> ProjectWorkspace {
        let packages: Vec<Package> = packages.into_iter().cloned().collect();
        let targets = ProjectWorkspace::bsp_targets_from_metadata_packages(&packages);
        ProjectWorkspace {
            _packages: packages,
            build_targets: targets
        }
    }

    fn bsp_targets_from_metadata_packages(packages: &[Package]) -> Vec<BuildTarget> {
        packages.iter()
            .flat_map(|package| package.targets.iter())
            .map(BuildTarget::from)
            .collect()
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