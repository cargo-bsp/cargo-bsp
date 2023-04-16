use crate::bsp_types::mappings::build_target::new_bsp_build_target;
use crate::bsp_types::BuildTarget;
use crate::project_model::package_dependencies::{PackageDependency, PackageWithDependencies};
use cargo_metadata::{CargoOpt, Error, MetadataCommand, Package};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    pub _cargo_targets: Vec<cargo_metadata::Target>,
    build_targets: Vec<BuildTarget>,
}

impl ProjectWorkspace {
    pub fn new(project_manifest_path: PathBuf) -> Result<ProjectWorkspace, Error> {
        let metadata = MetadataCommand::new()
            .manifest_path(project_manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec()?;

        let packages_with_dependencies: Vec<PackageWithDependencies> = metadata
            .workspace_packages()
            .iter()
            .map(|&package| {
                PackageWithDependencies(
                    package,
                    package
                        .dependencies
                        .iter()
                        .map(|dep| PackageDependency::new(dep, &metadata.packages))
                        .collect(),
                )
            })
            .collect();

        //packages and dependencies can be transformed Vec<(&Package, Vec<PathBuf>)> where PathBuf is project manifest to dep, if features won 't be needed
        let targets =
            ProjectWorkspace::bsp_targets_from_metadata_packages(&packages_with_dependencies);

        let workspace_packages = metadata.workspace_packages();
        let cargo_targets = ProjectWorkspace::cargo_targets(&workspace_packages);
        // let targets = ProjectWorkspace::bsp_targets_from_metadata_packages(&workspace_packages);

        Ok(ProjectWorkspace {
            _cargo_targets: cargo_targets,
            build_targets: targets,
        })
    }

    /// Return targets from all packages
    fn cargo_targets(packages: &[&Package]) -> Vec<cargo_metadata::Target> {
        packages
            .iter()
            .flat_map(|package| package.targets.iter())
            .cloned()
            .collect()
    }

    /// Create BSP build targets from cargo targets from all packages in the workspace
    fn bsp_targets_from_metadata_packages(
        packages_with_deps: &[PackageWithDependencies],
    ) -> Vec<BuildTarget> {
        packages_with_deps
            .iter()
            .flat_map(|PackageWithDependencies(p, d)| {
                p.targets.iter().map(|t| new_bsp_build_target(t, d))
            })
            .collect()
    }

    pub fn get_build_targets(&self) -> Vec<BuildTarget> {
        self.build_targets.clone()
    }
}
