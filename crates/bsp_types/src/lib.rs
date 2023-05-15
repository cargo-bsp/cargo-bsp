pub use basic_bsp_structures::*;

pub mod basic_bsp_structures;
pub mod requests;

pub mod notifications;

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