//! [`Check`] handles communication with Cargo regarding RustWorkspace Request that requires
//! information from `cargo check` command.

mod cargo_message_to_package_info;
mod check_actor;
mod check_handle;
