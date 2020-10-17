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

#[tokio::main]
async fn main() {
    let mut listener = net::TcpListener::bind("127.0.0.1:6142").await.unwrap();

    while let Some(socket_res) = listener.next().await {
        match socket_res {
            Ok(mut socket) => {
                println!("Accepted connection from {:?}", socket.peer_addr());
                let libsocks5 = load_library("socks5").unwrap();
                unsafe {
                    let socks5init: libloading::Symbol<unsafe extern fn(*const c_void, *const c_void, *const c_void)> = libsocks5.get(b"init\0").unwrap();
                    socks5init(sopipe_get as *const _, sopipe_set as *const _, sopipe_write as *const _);

                    let socks5version: libloading::Symbol<unsafe extern fn(*mut i32, *mut [u8])> = libsocks5.get(b"version\0").unwrap();
                    let mut len = 0;
                    let mut buf = [0; 40];
                    socks5version(&mut len, &mut buf);
                    println!("{}", std::str::from_utf8_unchecked(&buf[0..len as _]))
                }

                unsafe {
                    let forward: libloading::Symbol<unsafe extern fn(stream: *mut c_void, len: i32, buffer: *const u8)> = libsocks5.get(b"forward\0").unwrap();
                    let backward: libloading::Symbol<unsafe extern fn(stream: *mut c_void, len: i32, buffer: *const u8)> = libsocks5.get(b"backward\0").unwrap();

                    let mut buf = [0; 2048];
                    let forward_handle = async {
                        while let Ok(l) = socket.read(&mut buf).await {
                            if l == 0 {
                                break
                            }

                            let mut stream = SopipeStream::new();
                            forward(&mut stream as *mut _ as *mut _, l as _, buf.as_ptr());
                        }
                    };

                    forward_handle.await
                }

                io::copy(&mut socket, &mut io::stdout()).await.unwrap();
            }
            Err(err) => {
                println!("accept error = {:?}", err)
            }
        }
    }
}

struct SopipeStream {
    // socket: std::sync::Arc<std::sync::Mutex<dyn AsyncWrite>>,
    dict: std::sync::Arc<std::sync::Mutex<std::collections::BTreeMap<Box<[u8]>, Box<[u8]>>>>,
}

impl SopipeStream {
    fn new() -> Self {
        Self {
            dict: std::sync::Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()))
        }
    }
}

unsafe extern fn sopipe_get(stream: *mut SopipeStream, key_len: i32, key_ptr: *const u8, value_len: *mut i32, value_ptr: *mut u8) {
    let stream = &mut *stream;
    let key = core::slice::from_raw_parts(key_ptr, key_len as _);
    if let Some(value) = stream.dict.lock().unwrap().get(key) {
        if *value_len >= value.len() as _ {
            core::slice::from_raw_parts_mut(value_ptr, *value_len as _).copy_from_slice(value)
        }
        *value_len = value.len() as _;
    } else {
        *value_len = -1
    }
}

unsafe extern fn sopipe_set(stream: *mut SopipeStream, key_len: i32, key_ptr: *const u8, value_len: i32, value_ptr: *mut u8) {
    let stream = &mut *stream;
    let key = core::slice::from_raw_parts(key_ptr, key_len as _);
    if value_len < 0 {
        stream.dict.lock().unwrap().remove(key);
        return
    }

    let value = core::slice::from_raw_parts(value_ptr, value_len as _);
    stream.dict.lock().unwrap().insert(key.into(), value.into());
}

unsafe extern fn sopipe_write(stream: *mut SopipeStream, len: i32, data: *const u8) {
    let stream = &mut *stream;
    let buffer = core::slice::from_raw_parts(data, len as _);
    print!("write: {}", core::str::from_utf8_unchecked(buffer))
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
