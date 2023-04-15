use std::env;
use std::fs::File;

use simplelog::*;

use cargo_bsp::server;

pub fn main() -> server::Result<()> {
    let exe_path = env::current_exe().unwrap();
    let debug_dir = exe_path.parent().unwrap();
    let target_dir = debug_dir.parent().unwrap();
    let project_dir = target_dir.parent().unwrap();
    let log_file = project_dir.join("logs.log");

    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(log_file.to_str().unwrap()).unwrap(),
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
