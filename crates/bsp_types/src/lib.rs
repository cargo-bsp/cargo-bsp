//! Implementation of the [BSP structures](https://build-server-protocol.github.io/docs/specification) in Rust.

pub use basic_bsp_structures::*;

pub mod basic_bsp_structures;
pub mod requests;

pub mod extensions;
pub mod notifications;
pub mod rust_extension;

pub mod tests {
    use serde::Deserialize;

    pub fn test_deserialization<T>(json: &str, expected: &T)
    where
        T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let value = serde_json::from_str::<T>(json).unwrap();
        assert_eq!(&value, expected);
    }
}
