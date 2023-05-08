use std::io::prelude::*;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};

use serde_json::from_str;

use cargo_bsp::communication::Message;

pub struct Client<'a> {
    buf_reader: BufReader<&'a mut ChildStdout>,
    child_stdin: ChildStdin,
}

impl<'a> Client<'a> {
    pub fn new(child: &'a mut Child) -> Self {
        Self {
            buf_reader: BufReader::new(child.stdout.as_mut().unwrap()),
            child_stdin: child.stdin.take().unwrap(),
        }
    }

    fn add_headers(&self, message: &str) -> String {
        let mut prefix = format!("Content-Length: {}\r\n\r\n", message.len());
        prefix.push_str(message);
        prefix
    }

    fn parse_headers(&mut self) -> usize {
        let mut header = String::new();
        let mut content_length = 0;
        self.buf_reader
            .read_line(&mut header)
            .expect("Failed to read headers");

        let val = header.split(": ").collect::<Vec<&str>>()[1];
        if val.as_bytes()[0].is_ascii_digit() {
            content_length = from_str::<usize>(val).unwrap();
        };

        self.buf_reader
            .read_line(&mut header)
            .expect("Failed to read newlines");
        content_length
    }

    fn read_n_chars(&mut self, no_chars: usize) -> String {
        let mut buf = vec![0u8; no_chars];
        self.buf_reader
            .read_exact(&mut buf)
            .expect("Failed to read the actual content");
        String::from_utf8(buf).unwrap()
    }

    pub fn send(&mut self, msg: &str) {
        let msg_with_headers = self.add_headers(msg);
        self.child_stdin
            .write_all(msg_with_headers.as_bytes())
            .expect("Failed to send a message");
        println!("Client has send a message:\n{}", msg_with_headers);
    }

    pub fn recv_resp(&mut self) -> String {
        let msg_len = self.parse_headers();
        let mut msg = self.read_n_chars(msg_len);
        while let Ok(Message::Notification(notif)) = from_str(&msg) {
            println!("Client got a notification: {:?}\n", notif);
            let content_length = self.parse_headers();
            msg = self.read_n_chars(content_length);
        }
        match from_str(&msg) {
            Ok(Message::Response(resp)) => println!("Client got a response: {:?}\n", resp),
            _ => println!("Client got an invalid message: {}", msg),
        }
        msg
    }
}
