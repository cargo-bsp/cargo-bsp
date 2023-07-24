//! [`Server`] manages the server and communication with the client and routes
//! some requests to [`CargoCommunication`] and [`ProjectModel`].

use std::fmt;

use serde::de::DeserializeOwned;

pub use main_loop::main_loop;
pub use server_run::run_server;

pub(crate) mod caps;
pub(crate) mod config;
mod dispatch;
pub(crate) mod global_state;
mod handlers;
mod main_loop;
mod server_run;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn from_json<T: DeserializeOwned>(what: &'static str, json: &serde_json::Value) -> Result<T> {
    let res = serde_json::from_value(json.clone())
        .map_err(|e| format!("Failed to deserialize {}: {}; {}", what, e, json))?;
    Ok(res)
}

#[derive(Debug)]
pub struct LspError {
    code: i32,
    message: String,
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Language Server request failed with {}. ({})",
            self.code, self.message
        )
    }
}

impl std::error::Error for LspError {}
