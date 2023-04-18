use std::env;
use std::fs::File;
use std::path::PathBuf;

use simplelog::*;

use cargo_bsp::server;

#[cfg(debug_assertions)]
fn log_file_location() -> PathBuf {
    let exe_path = env::current_exe().unwrap();
    let debug_dir = exe_path.parent().unwrap();
    let target_dir = debug_dir.parent().unwrap();
    let project_dir = target_dir.parent().unwrap();
    project_dir.join("logs.log")
}

#[cfg(not(debug_assertions))]
fn log_file_location() -> PathBuf {
    let project_dir = env::current_dir().unwrap();
    project_dir.join("cargo-bsp.log")
}

pub fn main() -> server::Result<()> {
    // Setting logger configuration and logging files location
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(log_file_location().to_str().unwrap()).unwrap(),
        ),
        TermLogger::new(
            LevelFilter::Trace,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    ])
    .unwrap();

    server::run_server()
}
