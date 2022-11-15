use std::io::{stderr, stdout, Write};

pub fn send(message: &str) {
    stdout().write_all(message.as_bytes()).unwrap();
    stdout().write_all("\n".as_bytes()).unwrap();
}

pub fn log(message: &str) {
    stderr().write_all(message.as_bytes()).unwrap();
}