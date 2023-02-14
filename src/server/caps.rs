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
        inverse_sources_provider: None,
        dependency_sources_provider: None,
        dependency_modules_provider: None,
        resources_provider: None,
        output_paths_provider: None,
        build_target_changed_provider: None,
        jvm_run_environment_provider: None,
        jvm_test_environment_provider: None,
        can_reload: None,
    }
}