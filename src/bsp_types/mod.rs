pub use basic_bsp_structures::*;

mod basic_bsp_structures;
pub mod mappings;
pub mod notifications;
pub mod requests;

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    pub(crate) fn test_deserialization<T>(json: &str, expected: &T)
    where
        T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let value = serde_json::from_str::<T>(json).unwrap();
        assert_eq!(&value, expected);
    }
}
