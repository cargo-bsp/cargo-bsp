// copy from rust-analyzer

use crate::bsp_types::{BuildServerCapabilities, CompileProvider};
use crate::server::config::Config;

pub fn server_capabilities(config: &Config) -> BuildServerCapabilities {
    BuildServerCapabilities {
        compile_provider: Some(CompileProvider {
            language_ids: config.caps.language_ids.clone(),
        }),
        test_provider: None,
        run_provider: None,
        debug_provider: None,
        inverse_sources_provider: Some(false),
        dependency_sources_provider: Some(false),
        dependency_modules_provider: Some(false),
        resources_provider: Some(true),
        output_paths_provider: Some(false),
        build_target_changed_provider: Some(false),
        jvm_run_environment_provider: Some(false),
        jvm_test_environment_provider: Some(false),
        can_reload: Some(false),
    }
}