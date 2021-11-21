use std::process::Command;

#[test]
fn load() {
    println!("{:?}", String::from_utf8(Command::new(env!("CARGO_BIN_EXE_sopipe")).output().unwrap().stdout).unwrap());
}
