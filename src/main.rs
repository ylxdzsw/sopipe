#![allow(irrefutable_let_patterns)]
#![allow(dead_code, unused_imports)]
#![allow(non_camel_case_types)]
#![deny(bare_trait_objects)]
#![warn(clippy::all)]

use oh_my_rust::*;
use tokio::net;
use tokio::io;
use tokio::prelude::*;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() {
    let mut listener = net::TcpListener::bind("127.0.0.1:6142").await.unwrap();
    let mut incoming = listener.incoming();
    while let Some(socket_res) = incoming.next().await {
        match socket_res {
            Ok(mut socket) => {
                println!("Accepted connection from {:?}", socket.peer_addr());
                let libsocks5 = load_library("socks5").unwrap();
                unsafe {
                    let socks5version: libloading::Symbol<unsafe extern fn(*mut [u8]) -> i32> = libsocks5.get(b"version\0").unwrap();
                    let mut buf = [0; 40];
                    let len = socks5version(&mut buf);
                    println!("{}", std::str::from_utf8_unchecked(&buf[0..len as _]))
                }
                io::copy(&mut socket, &mut io::stdout()).await.unwrap();
            }
            Err(err) => {
                println!("accept error = {:?}", err)
            }
        }
    }
}

// TODO: a macro that loads a library and keeps the symbols in a struct
fn load_library(libname: &str) -> std::io::Result<libloading::Library> {
    let name = if cfg!(windows) {
        format!("lib{}.dll", libname)
    } else if cfg!(unix) {
        format!("lib{}.so", libname)
    } else {
        unimplemented!()
    };

    libloading::Library::new(name)
}
