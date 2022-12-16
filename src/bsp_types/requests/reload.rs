//dev: can be implemented using wrapper like in lsp_types crate

use crate::bsp_types::requests::Request;

#[derive(Debug)]
pub enum Reload {}

impl Request for Reload {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "workspace/reload";
}

/*
The reload request is sent from the client to instruct the build server to reload the build configuration.
This request should be supported by build tools that keep their state in memory.
If the reload request returns with an error, it's expected that other requests respond with the previously known "good" state.

Request:
method: workspace/reload
params: null

Response:
result: null
error: code and message in case an error happens during reload. For example, when the build configuration is invalid.
*/
