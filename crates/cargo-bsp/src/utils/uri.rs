//! Maps the path into URI.

use std::fmt::Display;

use bsp_types::URI;

pub fn file_uri<T: Display>(path: T) -> URI {
    format!("file://{}", path)
}
