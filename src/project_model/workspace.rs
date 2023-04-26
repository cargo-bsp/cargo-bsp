use crate::bsp_types::mappings::build_target::{
    bsp_build_target_from_cargo_target, build_target_id_from_name_and_path,
};
use crate::bsp_types::{BuildTarget, BuildTargetIdentifier};
use crate::project_model::package::BspPackage;
use crate::project_model::package_dependencies::{PackageDependency, PackageWithDependenciesIds};
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::{CargoOpt, Error, MetadataCommand, Target};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default, Debug)]
pub struct ProjectWorkspace {
    /// BSP build targets from all packages in the workspace
    pub build_targets: Vec<BuildTarget>,

    pub packages: Vec<BspPackage>,

    pub target_id_target_details_map: HashMap<BuildTargetIdentifier, Rc<cargo_metadata::Target>>,

    pub target_id_package_map: HashMap<BuildTargetIdentifier, String>,
}

pub struct CommandCallTargetDetails {
    pub name: String,
    pub kind: String,
    pub package_abs_path: Utf8PathBuf,
    pub enabled_features: Vec<String>,
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

        let (target_id_to_package, target_id_to_target_details) = ProjectWorkspace::create_hashmaps(&bsp_packages);

        // let target_map: HashMap<BuildTargetIdentifier, Rc<BspPackage>> = bsp_packages
        //     .iter()
        //     .flat_map(|p| -> Vec<(BuildTargetIdentifier, Rc<BspPackage>)> {
        //         let package = p.as_ref();
        //         package
        //             .targets
        //             .iter()
        //             .map(|t| {
        //                 (
        //                     //todo get easily acces to kind and name from target
        //                     // resign from RC, can't be mutated
        //                     build_target_id_from_name_and_path(&t.name, &t.src_path),
        //                     Rc::clone(p),
        //                 )
        //             })
        //             .collect()
        //     })
        //     .collect();

        // todo get target_info from bsp_packages while parsing to bsp_build_target

        Ok(ProjectWorkspace {
            build_targets: vec![],
            packages: bsp_packages,
            target_id_target_details_map: Default::default(),
            target_id_package_map: Default::default(),
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

    pub fn create_hashmaps(bsp_packages: &[BspPackage]) {
        let x: Vec<((BuildTargetIdentifier, String), (BuildTargetIdentifier, Rc<Target>))> = bsp_packages.iter().flat_map(|p|
            p.targets.iter().map(
                |tr|
                    (
                        //bid to package name map
                        (build_target_id_from_name_and_path(&tr.name, &tr.src_path), p.name.clone()),
                        //bid to target rc reference
                        (build_target_id_from_name_and_path(&tr.name, &tr.src_path), Rc::clone(tr))
                    )
            )
        ).collect();
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

    pub fn get_target_details_for_command_call(
        &self,
        id: &BuildTargetIdentifier,
    ) -> Option<CommandCallTargetDetails> {
        if let Some(rc_package) = self.target_id_package_map.get(id) {
            return Some(CommandCallTargetDetails {
                name: "raczej trzebacoś poprawić".to_string(),
                kind: "tuteż".to_string(),
                package_abs_path: Default::default(),
                enabled_features: Default::default(),
            });
        }
        None
    }
}
