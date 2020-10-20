#![allow(irrefutable_let_patterns)]
#![allow(dead_code, unused_imports)]
#![allow(non_camel_case_types)]
#![deny(bare_trait_objects)]
#![warn(clippy::all)]
#![allow(clippy::type_complexity)]

use oh_my_rust::*;
use tokio::net;
use tokio::io;
use tokio::prelude::*;
use futures::stream::StreamExt;
use core::ffi::c_void;

mod plugins;
use plugins::Plugin;

type ByteDict = std::collections::BTreeMap<Box<[u8]>, Box<[u8]>>;

#[tokio::main]
async fn main() {
    let mut listener = net::TcpListener::bind("127.0.0.1:6142").await.unwrap();

    let tcplistener = TcpListener::new(&vec![("port".to_owned(), "6142".to_owned())].into_iter().collect());
    let socks5 = unsafe { Plugin::load("socks5") };

    let pipeline = vec![Box::new(tcplistener as &dyn Component), Box::new(socks5 as &dyn Component)];

    println!("{}", unsafe { socks5.version() })


}

struct Actor {
    comp: &'static dyn Component,
    pipeline: &'static [Box<&'static dyn Component>],
    args: ByteDict, // TODO: use an append-only Arc linked list (tree) because they are more copied than read
    state: *mut c_void,
    index: usize // its index in the pipeline
}

impl Actor {

}

unsafe impl Send for Actor {}

#[async_trait::async_trait]
trait Component: 'static + Sync {
    async fn init(&'static self, args: &ByteDict) -> *mut c_void;
    async fn start(&'static self, actor: &mut Actor);
}

struct TcpListener {
    port: u32
}

impl TcpListener {
    fn new(args: &std::collections::BTreeMap<String, String>) -> &'static Self {
        leak(TcpListener {
            port: args["port"].parse().unwrap()
        })
    }
}

#[async_trait::async_trait]
impl Component for TcpListener {
    async fn init(&'static self, _args: &ByteDict) -> *mut c_void {
        let handle = net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await.unwrap();
        leak(handle) as *mut _ as _
    }

    async fn start(&'static self, actor: &mut Actor) {
        let mut listener = core::pin::Pin::new(unsafe { &mut *(actor.state as *mut net::TcpListener) });

        while let Some(socket_res) = listener.as_mut().next().await {
            match socket_res {
                Ok(mut socket) => {
                    println!("Accepted connection from {:?}", socket.peer_addr());

                    // unsafe {
                    //     let forward: libloading::Symbol<unsafe extern fn(stream: *mut c_void, len: i32, buffer: *const u8)> = libsocks5.get(b"forward\0").unwrap();
                    //     let backward: libloading::Symbol<unsafe extern fn(stream: *mut c_void, len: i32, buffer: *const u8)> = libsocks5.get(b"backward\0").unwrap();

                    //     let mut buf = [0; 2048];
                    //     let forward_handle = async {
                    //         while let Ok(l) = socket.read(&mut buf).await {
                    //             if l == 0 {
                    //                 break
                    //             }

                    //             let mut stream = SopipeStream::new();
                    //             forward(&mut stream as *mut _ as *mut _, l as _, buf.as_ptr());
                    //         }
                    //     };

                    //     forward_handle.await
                    // }

                    // io::copy(&mut socket, &mut io::stdout()).await.unwrap();
                }
                Err(err) => {
                    println!("accept error = {:?}", err)
                }
            }
        }
    }
}
