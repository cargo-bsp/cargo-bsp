use std::env;
use std::fs::{create_dir, File};
use std::path::PathBuf;

use simplelog::*;

use cargo_bsp::server;

#[cfg(debug_assertions)]
fn debug_log_file_path() -> PathBuf {
    let exe_path = env::current_exe().unwrap();
    let debug_dir = exe_path.parent().unwrap();
    let target_dir = debug_dir.parent().unwrap();
    let project_dir = target_dir.parent().unwrap();
    project_dir.join("server.log")
}

fn log_file_path() -> PathBuf {
    let logs_dir = env::current_dir().unwrap().join(".cargobsp");
    if !logs_dir.exists() {
        let _ = create_dir(logs_dir.clone()); // no unwrap, it messes up integration test
    }
    logs_dir.join("server.log")
}

pub fn main() -> server::Result<()> {
    // Setting logger configuration and logging files location
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(log_file_path().to_str().unwrap()).unwrap(),
        ),
        #[cfg(debug_assertions)]
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(debug_log_file_path().to_str().unwrap()).unwrap(),
        ),
        #[cfg(debug_assertions)]
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
