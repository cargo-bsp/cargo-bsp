use std::ops::Add;

use cargo_metadata::camino::Utf8Path;

use crate::bsp_types::basic_bsp_structures::*;

impl BuildTargetCapabilities {
    pub fn new() -> Self {
        BuildTargetCapabilities {
            can_compile: false,
            can_test: false,
            can_run: false,
            can_debug: false,
        }
    }

    pub fn enable_all(&mut self) {
        self.can_compile = true;
        self.can_test = true;
        self.can_run = true;
        self.can_debug = true;
    }

    pub fn enable_compile(&mut self) -> &mut Self {
        self.can_compile = true;
        self
    }

    pub fn enable_test(&mut self) -> &mut Self {
        self.can_test = true;
        self
    }

    pub fn enable_run(&mut self) -> &mut Self {
        self.can_run = true;
        self
    }

    pub fn enable_debug(&mut self) -> &mut Self {
        self.can_debug = true;
        self
    }
}

fn tags_and_capabilities_from_cargo_kind(
    cargo_target: &cargo_metadata::Target,
) -> (Vec<BuildTargetTag>, BuildTargetCapabilities) {
    let mut tags = vec![];
    let mut capabilities = BuildTargetCapabilities::new();
    cargo_target
        .kind
        .iter()
        .for_each(|kind| match kind.as_str() {
            "lib" => {
                tags.push(BuildTargetTag::Library);
                capabilities.enable_compile().enable_test().enable_debug();
            }
            "bin" => {
                tags.push(BuildTargetTag::Application);
                capabilities.enable_all();
            }
            "example" => {
                tags.push(BuildTargetTag::Application);
                capabilities.enable_compile().enable_run().enable_debug();
            }
            "test" => {
                tags.push(BuildTargetTag::Test);
                capabilities.enable_compile().enable_run().enable_debug();
            }
            "bench" => {
                tags.push(BuildTargetTag::Benchmark);
                capabilities.enable_compile().enable_run().enable_debug();
            }
            "custom-build" => {
                todo!()
            }
            _ => (),
        });

    (tags, capabilities)
}

fn discover_dependencies(_path: &Utf8Path) -> Vec<BuildTargetIdentifier> {
    vec![] //todo
}

impl From<&cargo_metadata::Target> for BuildTarget {
    fn from(cargo_target: &cargo_metadata::Target) -> Self {
        let (tags, capabilities) = tags_and_capabilities_from_cargo_kind(cargo_target);

        let mut base_directory = cargo_target.src_path.clone();
        base_directory.pop();

        BuildTarget {
            id: BuildTargetIdentifier {
                uri: cargo_target.src_path.to_string() + ":" + &cargo_target.name,
            },
            display_name: Some(cargo_target.name.clone()),
            base_directory: Some("file://".to_owned().add(base_directory.as_ref())),
            tags,
            capabilities,
            language_ids: vec![RUST_ID.to_string()],
            dependencies: discover_dependencies(&cargo_target.src_path),
            data_kind: Some("rust".to_string()),
            data: Some(RustBuildTarget {
                edition: cargo_target.edition.clone(),
                required_features: cargo_target.required_features.clone(),
            }),
        }
    }
}
