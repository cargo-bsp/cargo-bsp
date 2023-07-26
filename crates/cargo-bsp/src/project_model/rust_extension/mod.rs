//! This module ia an implementation of handling the BSP Rust extension.

mod package;
mod target;
mod toolchain;

pub use self::package::{get_rust_packages_related_to_targets, resolve_raw_dependencies};
pub use self::toolchain::get_rust_toolchains;

use bsp_types::extensions::RustEdition;
use cargo_metadata::Edition;

pub(crate) fn metadata_edition_to_rust_extension_edition(metadata_edition: Edition) -> RustEdition {
    match metadata_edition {
        Edition::E2015 => RustEdition::Edition2015,
        Edition::E2018 => RustEdition::Edition2018,
        _ => RustEdition::Edition2021,
    }
}
