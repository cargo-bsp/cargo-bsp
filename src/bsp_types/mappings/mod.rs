use crate::bsp_types::Uri;
use std::fmt::Display;

pub mod to_publish_diagnostics;

pub fn file_uri<T: Display>(path: T) -> Uri {
    format!("file://{}", path)
}
