use crate::bsp_types::mappings::build_target::new_bsp_build_target;
use crate::bsp_types::BuildTarget;
use crate::project_model::package_dependencies::{PackageDependency, PackageWithDependenciesIds};
use cargo_metadata::{CargoOpt, Error, MetadataCommand, Package};
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    /// cargo_metadata targets from all packages in the workspace
    pub _cargo_targets: Vec<cargo_metadata::Target>,
    /// BSP build targets from all packages in the workspace
    pub build_targets: Vec<BuildTarget>,
}

impl ProjectWorkspace {
    /// Retrieves build targets from *'cargo metadata'*, maps them to BSP build
    /// targets and stores in new instance of ProjectWorkspace.
    ///
    /// Skips unit_tests discovery, see:
    /// [get_unit_tests_build_targets](crate::project_model::_unit_tests_discovery::get_unit_tests_build_targets).
    pub fn new(project_manifest_path: PathBuf) -> Result<ProjectWorkspace, Error> {
        let metadata = MetadataCommand::new()
            .manifest_path(project_manifest_path)
            .features(CargoOpt::AllFeatures) // TODO: in future can be a way to discover dependencies with features
            .exec()?;

        let workspace_packages = metadata.workspace_packages();
        let cargo_targets = ProjectWorkspace::cargo_targets(&workspace_packages);

        let packages_with_dependencies: Vec<PackageWithDependenciesIds> = workspace_packages
            .iter()
            .map(|&package| {
                PackageWithDependenciesIds(
                    package,
                    package
                        .dependencies
                        .iter()
                        .filter_map(|dep| {
                            PackageDependency::new(dep, &metadata.packages)?
                                .create_id_from_dependency()
                        })
                        .collect(),
                )
            })
            .collect();

        let targets =
            ProjectWorkspace::bsp_targets_from_metadata_packages(packages_with_dependencies);

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
        packages_with_deps: Vec<PackageWithDependenciesIds>,
    ) -> Vec<BuildTarget> {
        packages_with_deps
            .into_iter()
            .flat_map(|PackageWithDependenciesIds(p, d)| {
                p.targets.iter().map(move |t| new_bsp_build_target(t, &d))
            })
            .collect()
    }

    pub fn get_build_targets(&self) -> Vec<BuildTarget> {
        self.build_targets.clone()
    }
}
