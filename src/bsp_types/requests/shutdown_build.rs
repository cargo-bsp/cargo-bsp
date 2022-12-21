use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum ShutdownBuild {}

impl Request for ShutdownBuild {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "build/shutdown";
}
