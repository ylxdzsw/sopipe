use oh_my_rust::*;
use std::process::Command;

#[test]
fn load() {
    println!("{:?}", String::from_utf8(Command::new(env!("CARGO_BIN_EXE_sopipe")).output().unwrap().stdout).unwrap());
}

// "tcp(2222) -> socks5_server() -> chacha20_encode() !! chacha20_decode() -> send()"
