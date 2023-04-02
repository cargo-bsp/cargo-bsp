pub use basic_bsp_structures::*;

mod basic_bsp_structures;
pub mod requests;

pub mod mappings;
pub mod notifications;
pub mod cargo_output;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    pub(crate) fn test_serialization<SER>(ms: &SER, expected: &str)
    where
        SER: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json_str = serde_json::to_string(ms).unwrap();
        assert_eq!(&json_str, expected);
        test_deserialization(&json_str, ms);
    }

    pub(crate) fn test_deserialization<T>(json: &str, expected: &T)
    where
        T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let value = serde_json::from_str::<T>(json).unwrap();
        assert_eq!(&value, expected);
    }
}
