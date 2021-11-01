#![feature(new_uninit)]
#![feature(box_into_pin)]
#![feature(never_type)]

#![allow(clippy::mut_from_ref)] // to many false positives

use oh_my_rust::*;

use anyhow::{Context, Result};

// expose to components
pub use tokio::net;
pub use tokio::io;

type SVec<T> = smallvec::SmallVec<[T; 3]>;

// mod endpoints;
// use endpoints::*;

mod script;

pub use api::*;

// mod plugins;
// use plugins::Plugin;

#[allow(clippy::vec_init_then_push)]
fn main() -> Result<!> {
    // let tcplistener = TcpListener::new(&vec![("port".to_owned(), "6142".to_owned())].into_iter().collect());
    // let socks5 = unsafe { Plugin::load("socks5") };

    // let pipeline = vec![Box::new(tcplistener as &dyn Component), Box::new(socks5 as &dyn Component)];

    // // println!("{}", unsafe { socks5.version() })

    // let source_actor = Actor {
    //     comp: *pipeline[0],
    //     args: std::collections::BTreeMap::new(),
    //     state: tcplistener.creat_forward_context("".as_bytes()),
    //     index: 0,
    //     direction: Direction::Forward
    // };

    // unsafe {
    // let library =  leak(libloading::Library::new("libxor.so").unwrap());

    // let hello: libloading::Symbol<'static, unsafe extern fn()> = library.get(b"hello\0").unwrap();

    // hello();}

    let mut components = vec![];

    #[cfg(feature="xor")]
    components.push(xor::init());

    #[cfg(feature="tcp")]
    components.push(tcp::init());

    let rt = tokio::runtime::Runtime::new()?;

    script::load_script(r#"tcp(2222) => xor("fuck") => xor()"#, &components).unwrap();

    println!("{:?}", 42);
    unreachable!();
}

