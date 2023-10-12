//! Maps the path into URI.

use std::fmt::Display;

use bsp4rs::bsp::URI;

pub fn file_uri<T: Display>(path: T) -> URI {
    URI(format!("file://{}", path))
}
