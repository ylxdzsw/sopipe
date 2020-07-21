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
                io::copy(&mut socket, &mut io::stdout()).await.unwrap();
            }
            Err(err) => {
                println!("accept error = {:?}", err)
            }
        }
    }
}
