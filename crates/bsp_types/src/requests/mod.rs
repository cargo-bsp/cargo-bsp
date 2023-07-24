use serde::de::DeserializeOwned;
use serde::Serialize;

pub use build_target_sources::*;
pub use cargo_extension::*;
pub use clean_cache::*;
pub use compile::*;
pub use debug::*;
pub use dependency_modules::*;
pub use dependency_sources::*;
pub use initialize::*;
pub use inverse_sources::*;
pub use java_extension::*;
pub use output_paths::*;
pub use reload::*;
pub use resources::*;
pub use run::*;
pub use shutdown_build::*;
pub use test::*;
pub use workspace_build_targets::*;

pub mod cargo_extension;

mod build_target_sources;
mod clean_cache;
mod compile;
mod debug;
mod dependency_modules;
mod dependency_sources;
mod initialize;
mod inverse_sources;
mod java_extension;
mod output_paths;
mod reload;
mod resources;
mod run;
mod shutdown_build;
mod test;
mod workspace_build_targets;

pub trait Request {
    type Params: DeserializeOwned + Serialize;
    type Result: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}
