//! Maps the path into URI.

use std::fmt::Display;

use bsp_types::Uri;

pub fn file_uri<T: Display>(path: T) -> Uri {
    format!("file://{}", path)
}
