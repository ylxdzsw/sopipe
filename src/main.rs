#![feature(new_uninit)]
#![feature(box_into_pin)]
#![feature(never_type)]

#![allow(clippy::mut_from_ref)] // to many false positives

use std::collections::BTreeSet;

use api::Actor;
use oh_my_rust::*;

use anyhow::{Context, Result};

type SVec<T> = smallvec::SmallVec<[T; 3]>;

mod script;
mod runtime;

/// A component with runtime-tracked states
struct Node {
    comp: &'static dyn api::Component,
    outputs: &'static [usize],
    conj: usize
}


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

    let nodes: &_ = script::load_script(r#"tcp(2222) => xor("fuck") => xor()"#, &components).unwrap().leak();

    let runtime = runtime::Runtime::new(nodes).box_and_leak();

    let tokio_rt = tokio::runtime::Runtime::new()?;

    tokio_rt.block_on(async move {
        let non_source: BTreeSet<_> = nodes.iter()
            .flat_map(|x| x.outputs.iter())
            .copied().collect();
        let tasks: Vec<_> = nodes.iter().enumerate()
            .filter(|(i, _)| !non_source.contains(i))
            .map(|(_, x)| runtime.spawn(x))
            .map(|actor| tokio::spawn(actor.run()))
            .collect();
        for task in tasks {
            task.await.unwrap()
        }
    });

    unreachable!();
}

