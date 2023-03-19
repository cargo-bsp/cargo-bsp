pub use basic_bsp_structures::*;

mod basic_bsp_structures;
pub mod requests;

pub mod mappings;
pub mod notifications;

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    pub(crate) fn test_serialization<SER>(ms: &SER, expected: &str)
    where
        SER: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json_str = serde_json::to_string(ms).unwrap();
        assert_eq!(&json_str, expected);
        let deserialized: SER = serde_json::from_str(&json_str).unwrap();
        assert_eq!(&deserialized, ms);
    }
}
