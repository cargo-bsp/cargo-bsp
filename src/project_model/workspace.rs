use crate::bsp_types::mappings::build_target::{
    bsp_build_target_from_cargo_target, build_target_id_from_name_and_path,
};
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::project_model::package::BspPackage;
use crate::project_model::package_dependencies::{PackageDependency, PackageWithDependenciesIds};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, Error, MetadataCommand, Target};
use std::collections::HashMap;

use log::error;
use std::path::PathBuf;
use std::rc::Rc;

pub type BuildIdToPackageMap = HashMap<BuildTargetIdentifier, String>;
pub type BuildIdToTargetMap = HashMap<BuildTargetIdentifier, Rc<cargo_metadata::Target>>;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    /// BSP build targets from all packages in the workspace
    pub build_targets: Vec<BuildTarget>,

    pub packages: Vec<BspPackage>,

    pub target_id_target_details_map: BuildIdToTargetMap,

    pub target_id_package_map: BuildIdToPackageMap,
}

#[derive(Debug, Default, Clone)]
pub struct CommandCallTargetDetails<'a> {
    pub name: String,
    pub kind: String,
    pub package_abs_path: Utf8PathBuf,
    pub enabled_features: &'a [String],
}

impl ProjectWorkspace {
    /// Creates new ProjectWorkspace instance by retrieving following data from *'cargo metadata'*:
    /// * cargo_metadata crate targets, which are later mapped to BSP build targets,
    /// * features
    ///
    /// Skips unit_tests discovery, see:
    /// [get_unit_tests_build_targets](crate::project_model::_unit_tests_discovery::get_unit_tests_build_targets).
    pub fn new(project_manifest_path: PathBuf) -> Result<ProjectWorkspace, Error> {
        // We call it with --all-features, so we can get all features because we want
        // the output to contain all the packages - even those feature-dependent
        let metadata = MetadataCommand::new()
            .manifest_path(project_manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec()?;

        let workspace_packages = metadata.workspace_packages();

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
            build_targets: targets,
            packages: Default::default(),
            target_id_target_details_map: Default::default(),
            target_id_package_map: Default::default(),
        })
    }

    pub fn new2(project_manifest_path: PathBuf) -> Result<ProjectWorkspace, Error> {
        // We call it with --all-features, so we can get all features because we want
        // the output to contain all the packages - even those feature-dependent
        let metadata = MetadataCommand::new()
            .manifest_path(project_manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec()?;

        let bsp_packages: Vec<BspPackage> = metadata
            .workspace_packages()
            .into_iter()
            .map(|p| BspPackage::new(p, &metadata.packages))
            .collect();

        // let (target_id_to_package, target_id_to_target_details) = ProjectWorkspace::create_hashmaps(&bsp_packages);
        let (bid_to_package, bid_to_target) = ProjectWorkspace::create_hashmaps(&bsp_packages);

        // todo get target_info from bsp_packages while parsing to bsp_build_target

        Ok(ProjectWorkspace {
            build_targets: vec![],
            packages: bsp_packages,
            target_id_target_details_map: bid_to_target,
            target_id_package_map: bid_to_package,
        })
    }

    /// Create BSP build targets from cargo targets from all packages in the workspace
    fn bsp_targets_from_metadata_packages(
        packages_with_deps: Vec<PackageWithDependenciesIds>,
    ) -> Vec<BuildTarget> {
        packages_with_deps
            .into_iter()
            .flat_map(|PackageWithDependenciesIds(p, d)| {
                p.targets
                    .iter()
                    .map(move |t| bsp_build_target_from_cargo_target(t, &d))
            })
            .collect()
    }

    pub fn create_hashmaps(
        bsp_packages: &[BspPackage],
    ) -> (BuildIdToPackageMap, BuildIdToTargetMap) {
        bsp_packages
            .iter()
            .flat_map(|p| {
                p.targets.iter().map(|tr| {
                    let bid = build_target_id_from_name_and_path(&tr.name, &tr.src_path);
                    (
                        // BuildTargetIdentifier to package name map (key, value)
                        (bid.clone(), p.name.clone()),
                        // BuildTargetIdentifier to target_details map (key, value)
                        (bid, Rc::clone(tr)),
                    )
                })
            })
            .unzip()
    }

    pub fn get_build_targets(&self) -> Vec<BuildTarget> {
        self.build_targets.clone()
    }

    pub fn get_bsp_build_targets(&self) -> Vec<BuildTarget> {
        self.packages
            .iter()
            .flat_map(|p| p.get_bsp_build_targets())
            .collect()
    }

    pub fn find_build_target_package(
        &self,
        target_id: &BuildTargetIdentifier,
    ) -> Option<&BspPackage> {
        let package_name = self.target_id_package_map.get(target_id).or_else(|| {
            error!("Package not found for target: {:?}", target_id);
            None
        })?;

        self.packages
            .iter()
            .find(|p| p.name == *package_name)
            .or_else(|| {
                error!("Package not found for target: {:?}", target_id);
                None
            })
    }

    pub fn find_build_target_details(
        &self,
        target_id: &BuildTargetIdentifier,
    ) -> Option<&Rc<Target>> {
        self.target_id_target_details_map
            .get(target_id)
            .or_else(|| {
                error!("Target details not found for id: {:?}", target_id);
                None
            })
    }

    pub fn get_target_details_for_command_call(
        &self,
        id: &BuildTargetIdentifier,
    ) -> Option<CommandCallTargetDetails> {
        let mut target_data = CommandCallTargetDetails::default();

        let package = self.find_build_target_package(id)?;
        target_data.package_abs_path = package.manifest_path.clone();
        target_data.enabled_features = package.enabled_features.as_slice();

        let target_details = self.find_build_target_details(id)?;
        target_data.name = target_details.name.clone();
        target_data.kind = target_details
            .kind
            .get(0)
            .or_else(|| {
                error!("Invalid `kind vector` for target: {:?}", id);
                None
            })?
            .clone();

        Some(target_data)
    }
}
