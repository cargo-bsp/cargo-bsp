
use cargo_metadata::Package;

#[derive(Default)]
pub struct ProjectWorkspace {
    _packages: Vec<Package>,
}


// impl ProjectWorkspace {
//
//     pub fn new() -> ProjectWorkspace {
//         ProjectWorkspace {}
//     }
//
//     // pub fn new(packages: Vec<&Package>) -> ProjectWorkspace {
//     //     ProjectWorkspace {
//     //         packages,
//     //     }
//     // }
//
//     // pub fn add_packages(&mut self, packages: Vec<&Package> ) {
//     //     // TODO
//     // }
//
// }