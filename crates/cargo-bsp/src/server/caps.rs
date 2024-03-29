//! Sets the server capabilities for the client.

use bsp4rs::bsp::{BuildServerCapabilities, CompileProvider, RunProvider, TestProvider};

use crate::server::config::Config;

pub fn server_capabilities(config: &Config) -> BuildServerCapabilities {
    BuildServerCapabilities {
        compile_provider: Some(CompileProvider {
            language_ids: config.caps.language_ids.clone(),
        }),
        test_provider: Some(TestProvider {
            language_ids: config.caps.language_ids.clone(),
        }),
        run_provider: Some(RunProvider {
            language_ids: config.caps.language_ids.clone(),
        }),
        // This server does support BSP Cargo extension, but the client does not yet expect this field to appear.
        cargo_features_provider: None,
        debug_provider: None,
        inverse_sources_provider: Some(false),
        dependency_sources_provider: Some(false),
        dependency_modules_provider: Some(false),
        resources_provider: Some(false),
        output_paths_provider: Some(false),
        build_target_changed_provider: Some(false),
        jvm_run_environment_provider: Some(false),
        jvm_test_environment_provider: Some(false),
        can_reload: Some(true),
    }
}
