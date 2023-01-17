// temporary solution for logging
use std::io::{stderr, Write};

pub fn log(message: &str) {
    stderr().write_all(message.as_bytes()).unwrap();
    stderr().write_all("\n".as_bytes()).unwrap()
}
