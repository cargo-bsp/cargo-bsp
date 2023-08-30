//! [`CargoCheck`] manages spawning and handling information from `cargo check`
//! command needed for `RustWorkspace` request.

mod cargo_message_to_package_info;
mod check_actor;
mod check_handle;
