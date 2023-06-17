use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use cargo_metadata::{CargoOpt, Error, MetadataCommand};
use log::error;

use bsp_types::requests::cargo_extension::{Feature, PackageFeatures};
use bsp_types::{BuildTarget, BuildTargetIdentifier};

use crate::project_model::build_target_mappings::build_target_id_from_name_and_path;
use crate::project_model::cargo_package::CargoPackage;
use crate::project_model::target_details::TargetDetails;

pub type TargetIdToPackageName = HashMap<BuildTargetIdentifier, String>;
pub type TargetIdToTargetData = HashMap<BuildTargetIdentifier, Rc<cargo_metadata::Target>>;

#[derive(Default, Debug, Clone)]
pub struct ProjectWorkspace {
    /// List of all packages in a workspace
    pub packages: Vec<CargoPackage>,

    /// Map creating an easy access from BuildTargetIdentifier of a target to package name
    pub target_id_to_package_name: TargetIdToPackageName,

    /// Map creating an easy access from BuildTargetIdentifier of a target to its details
    pub target_id_to_target_data: TargetIdToTargetData,
}

impl ProjectWorkspace {
    /// Creates new ProjectWorkspace instance by extracting from *'cargo metadata'* workspace packages and its:
    /// * dependencies
    /// * targets
    /// * features
    ///
    /// Skips unit_tests discovery, see: [get_unit_tests_build_targets](crate::project_model::_unit_tests_discovery::get_unit_tests_build_targets).
    pub fn new(project_manifest_path: PathBuf) -> Result<ProjectWorkspace, Error> {
        // Cargo metadata is called with `--all-features`, so we can get all features because
        // we want the output to contain all the packages - even those feature-dependent
        let metadata = MetadataCommand::new()
            .manifest_path(project_manifest_path)
            .features(CargoOpt::AllFeatures)
            .exec()?;

        let bsp_packages: Vec<CargoPackage> = metadata
            .workspace_packages()
            .into_iter()
            .map(|p| CargoPackage::new(p, &metadata.packages))
            .collect();

        let (target_id_to_package_name, target_id_to_target_data) =
            ProjectWorkspace::create_hashmaps(&bsp_packages);

        Ok(ProjectWorkspace {
            packages: bsp_packages,
            target_id_to_package_name,
            target_id_to_target_data,
        })
    }

    fn create_hashmaps(
        bsp_packages: &[CargoPackage],
    ) -> (TargetIdToPackageName, TargetIdToTargetData) {
        bsp_packages
            .iter()
            .flat_map(|p| {
                p.targets.iter().map(|tr| {
                    let target_id = build_target_id_from_name_and_path(&tr.name, &tr.src_path);
                    (
                        // BuildTargetIdentifier to package name map (key, value)
                        (target_id.clone(), p.name.clone()),
                        // BuildTargetIdentifier to target_details map (key, value)
                        (target_id, Rc::clone(tr)),
                    )
                })
            })
            .unzip()
    }

    fn get_package_related_to_target(
        &self,
        target_id: &BuildTargetIdentifier,
    ) -> Option<&CargoPackage> {
        let package_name = self.target_id_to_package_name.get(target_id).or_else(|| {
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

    fn get_target_data(
        &self,
        target_id: &BuildTargetIdentifier,
    ) -> Option<&Rc<cargo_metadata::Target>> {
        self.target_id_to_target_data.get(target_id).or_else(|| {
            error!("Target details not found for id: {:?}", target_id);
            None
        })
    }

    /// Returns a list of all BSP build targets in a workspace
    pub fn get_bsp_build_targets(&self) -> Vec<BuildTarget> {
        self.packages
            .iter()
            .flat_map(|p| p.get_bsp_build_targets())
            .collect()
    }

    /// Returns target details for a given build target identifier
    pub fn get_target_details(&self, id: &BuildTargetIdentifier) -> Option<TargetDetails> {
        let package = self.get_package_related_to_target(id)?;
        let target_data = self.get_target_data(id)?;
        TargetDetails::new(package, target_data)
    }

    /// Returns a list of all packages with corresponding
    /// to them targets (as build target ids) and features
    pub fn get_cargo_features_state(&self) -> Vec<PackageFeatures> {
        self.packages
            .iter()
            .map(|p| p.get_enabled_features())
            .collect()
    }

    /// Enables features for a given package
    pub fn enable_features_for_package(&mut self, package_id: String, features: &[Feature]) {
        let package = self.packages.iter_mut().find(|p| p.id == package_id);
        if let Some(package) = package {
            package.enable_features(features);
        } else {
            error!(
                "Couldn't enable features, package not found for id: {:?}",
                package_id
            );
        }
    }

    /// Disables features for a given package
    pub fn disable_features_for_package(&mut self, package_id: String, features: &[Feature]) {
        let package = self.packages.iter_mut().find(|p| p.id == package_id);
        if let Some(package) = package {
            package.disable_features(features);
        } else {
            error!(
                "Couldn't disable features, package not found for id: {:?}",
                package_id
            );
        }
    }
}
