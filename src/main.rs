#![feature(new_uninit)]
#![feature(box_into_pin)]
#![feature(never_type)]

use oh_my_rust::*;

use anyhow::{Context, Result};

// expose to components
pub use tokio::net;
pub use tokio::io;

type SVec<T> = smallvec::SmallVec<[T; 3]>;

// mod endpoints;
// use endpoints::*;

mod script;
use script::*;

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

    println!("{:?}", 42);
    unreachable!();
}


// Additional notes:
// Read: the buffer will be released when it return, unless it is reused in the request that returned.
// Write: the buffer sent should be allocated by sopipe, and the ownership will be transfered back to sopipe. It can be the result of Alloc or Read requests.

// struct Actor {
//     comp: &'static Node,
//     state: *mut c_void,
// }

// trait Actor: Send {
//     fn poll(&'static self, arg: *mut c_void) -> Request;
// }

// enum InstanceState {} // opaque type to prevent mixing the pointer with other types

// trait Component: 'static + Sync {
//     /// create an instance of the component, return a pointer to the state. args includes the following pairs:
//     /// direction (String): either "forward" or "backward"
//     /// n_outputs (usize): the number of outputs
//     fn create(&'static self, args: &rhai::Map) -> *const InstanceState;

//     /// spawn an actor for a given instance
//     #[allow(clippy::mut_from_ref)]
//     fn spawn(&'static self, instance: *const InstanceState) -> &'static mut dyn Actor;

// }

