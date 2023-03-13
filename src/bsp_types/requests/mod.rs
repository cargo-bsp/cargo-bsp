use std::process::Command;
use serde::de::DeserializeOwned;
use serde::Serialize;

mod initialize;
pub use initialize::*;

mod build_target_sources;
pub use build_target_sources::*;

mod reload;
pub use reload::*;

mod shutdown_build;
pub use shutdown_build::*;

mod inverse_sources;
pub use inverse_sources::*;

mod dependency_sources;
pub use dependency_sources::*;

mod dependency_modules;
pub use dependency_modules::*;

mod resources;
pub use resources::*;

mod output_paths;
pub use output_paths::*;

mod compile;
pub use compile::*;

mod test;
pub use test::*;

mod run;
pub use run::*;

mod debug;
pub use debug::*;

mod clean_cache;
pub use clean_cache::*;

mod workspace_build_targets;
pub use workspace_build_targets::*;

mod java_extension;
pub use java_extension::*;

use crate::bsp_types::OriginId;

pub trait Request {
    type Params: DeserializeOwned + Serialize;
    type Result: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

pub trait CreateCommand {
    fn origin_id(&self) -> Option<OriginId>;

    fn create_command(&self) -> Command;
}
