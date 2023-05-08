use crate::project_model::package::Feature;
use cargo_metadata::camino::Utf8PathBuf;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
pub struct TargetDetails<'a> {
    pub name: String,
    pub kind: CargoTargetKind,
    pub package_abs_path: Utf8PathBuf,
    pub default_features_disabled: bool,
    pub enabled_features: &'a [Feature],
}

impl TargetDetails<'_> {
    pub fn set_kind(&mut self, kind: &str) {
        self.kind = serde_json::from_str::<CargoTargetKind>(kind).unwrap_or_else(|_| {
            error!("Invalid target kind: {}", kind);
            CargoTargetKind::default()
        });
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub enum CargoTargetKind {
    #[default]
    Lib,
    Bin,
    Example,
    Test,
    Bench,
}

impl ToString for CargoTargetKind {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
