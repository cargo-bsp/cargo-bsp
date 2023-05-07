use crate::bsp_types::mappings::build_target::{build_target_id_from_name_and_path, parent_path};
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::project_model::package::BspPackage;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, Error, MetadataCommand};
use std::collections::HashMap;

use log::error;
use std::path::PathBuf;
use std::rc::Rc;

pub type TargetIdToPackageNameMap = HashMap<BuildTargetIdentifier, String>;
pub type TargetIdToTargetDataMap = HashMap<BuildTargetIdentifier, Rc<cargo_metadata::Target>>;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    /// List of all packages in a workspace
    pub packages: Vec<BspPackage>,

    /// Map creating an easy access to package from BuildTargetIdentifier of a target
    pub target_id_package_map: TargetIdToPackageNameMap,

    /// Map creating an easy access from BuildTargetIdentifier of a target to its details
    pub target_id_target_details_map: TargetIdToTargetDataMap,
}

#[derive(Debug, Default, Clone)]
pub struct TargetDetails<'a> {
    pub name: String,
    pub kind: String,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: &'a [String],
}

impl ProjectWorkspace {
    /// Creates new ProjectWorkspace instance by extracting from *'cargo metadata'* workspace packages and its:
    /// * dependencies
    /// * targets
    /// * features
    /// [get_unit_tests_build_targets](crate::project_model::_unit_tests_discovery::get_unit_tests_build_targets).
    pub fn new(project_manifest_path: PathBuf) -> Result<ProjectWorkspace, Error> {
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

        let (bid_to_package_map, bid_to_target_map) =
            ProjectWorkspace::create_hashmaps(&bsp_packages);

        Ok(ProjectWorkspace {
            packages: bsp_packages,
            target_id_package_map: bid_to_package_map,
            target_id_target_details_map: bid_to_target_map,
        })
    }

    fn create_hashmaps(
        bsp_packages: &[BspPackage],
    ) -> (TargetIdToPackageNameMap, TargetIdToTargetDataMap) {
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

    fn find_build_target_package_in_map(
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

    fn find_build_target_details_in_map(
        &self,
        target_id: &BuildTargetIdentifier,
    ) -> Option<&Rc<cargo_metadata::Target>> {
        self.target_id_target_details_map
            .get(target_id)
            .or_else(|| {
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
    pub fn _get_target_details(&self, id: &BuildTargetIdentifier) -> Option<TargetDetails> {
        let mut target_data = TargetDetails::default();

        let package = self.find_build_target_package_in_map(id)?;
        target_data.package_abs_path = parent_path(&package.manifest_path);
        target_data.enabled_features = package.enabled_features.as_slice();
        target_data.default_features_disabled = package.default_features_disabled;

        let target_details = self.find_build_target_details_in_map(id)?;
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
