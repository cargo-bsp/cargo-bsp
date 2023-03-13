use serde::{Deserialize, Serialize};

mod basic_bsp_structures;
pub use basic_bsp_structures::*;

pub mod requests;

pub mod notifications;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct OriginId(String);

impl From<String> for OriginId {
    fn from(id: String) -> OriginId {
        OriginId(id)
    }
}