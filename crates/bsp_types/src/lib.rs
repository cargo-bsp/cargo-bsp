use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub mod bazel;
pub mod bsp;
pub mod cancel;
pub mod cargo;
pub mod rust;

use bazel::*;
use bsp::*;
use cancel::*;
use cargo::*;
use rust::*;

pub const PROTOCOL_VERSION: &str = "2.1.0";

pub trait Request {
    type Params: DeserializeOwned + Serialize;
    type Result: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

pub trait Notification {
    type Params: DeserializeOwned + Serialize;
    const METHOD: &'static str;
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherData {
    pub data_kind: String,
    pub data: serde_json::Value,
}

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
