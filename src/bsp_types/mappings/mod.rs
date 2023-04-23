use crate::bsp_types::Uri;
use std::fmt::Display;

pub mod build_target;

pub mod to_publish_diagnostics;

pub mod test;

pub fn file_uri<T: Display>(path: T) -> Uri {
    format!("file://{}", path)
}
