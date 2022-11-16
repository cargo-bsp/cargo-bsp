use std::io::{stderr, Write};

pub fn send(message: &str) {
    println!("{}", message);
}

pub fn log(message: &str) {
    stderr().write_all(message.as_bytes()).unwrap();
}
