use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum Reload {}

impl Request for Reload {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "workspace/reload";
}
