use cargo_metadata::camino::Utf8PathBuf;
use log::error;

#[derive(Debug, Default, Clone)]
pub struct TargetDetails<'a> {
    pub name: String,
    pub kind: TargetCargoKind,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: &'a [String],
}

impl TargetDetails<'_> {
    pub fn set_kind(&mut self, kind: &str) {
        self.kind = match kind {
            "lib" => TargetCargoKind::Lib,
            "bin" => TargetCargoKind::Bin,
            "example" => TargetCargoKind::Example,
            "test" => TargetCargoKind::Test,
            "bench" => TargetCargoKind::Bench,
            _ => {
                error!("Invalid target kind: {}", kind);
                TargetCargoKind::Lib
            }
        };
    }
}

#[derive(Debug, Default, Clone)]
pub enum TargetCargoKind {
    #[default]
    Lib,
    Bin,
    Example,
    Test,
    Bench,
}

impl ToString for TargetCargoKind {
    fn to_string(&self) -> String {
        match self {
            TargetCargoKind::Lib => "lib".to_string(),
            TargetCargoKind::Bin => "bin".to_string(),
            TargetCargoKind::Example => "example".to_string(),
            TargetCargoKind::Test => "test".to_string(),
            TargetCargoKind::Bench => "bench".to_string(),
        }
    }
}
