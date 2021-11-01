use oh_my_rust::*;
use std::{error::Error, ptr::NonNull};

use api::Argument;
use thiserror::Error;

struct Actor {

}

struct Component {

}

struct State {
    key: Option<Box<[u8]>>
}

#[derive(Error, Debug)]
pub enum XorError {
    #[error("data store disconnected")]
    Disconnect(#[from] std::io::Error),

    #[error("Invalid arguments. Detail: {0}")]
    InvalidArgument(&'static str),

    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader {
        expected: String,
        found: String,
    },
}

#[api::async_trait]
impl api::Actor for Actor {
    async fn feed(&mut self, ) {}
}

impl api::Component for Component {
    fn create(&self, arguments: Vec<Argument>) -> Result<NonNull<()>, Box<dyn Error + Send + Sync>> {
        let mut key = None;
        task!("creating instance of xor");
        for arg in arguments.into_iter() {
            let Argument(name, value) = arg;
            info!("{}: {:?}", name, value);
            if name.is_empty() || name == "key" {
                if key.is_none() {
                    match value {
                        api::ArgumentValue::String(x) => key = Some(x.into_bytes().into_boxed_slice()),
                        api::ArgumentValue::Int(_) => return Err(Box::new(XorError::InvalidArgument("key should be string"))),
                    }
                } else {
                    return Err(Box::new(XorError::InvalidArgument("key set twice")))
                }
            }
        }
        let state = State { key };
        Ok(NonNull::from(state.box_and_leak()).cast())
    }

    fn functions(&self) -> &'static [&'static str] {
        &["xor"]
    }

    fn spawn(&self, node_state: *const ()) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        let _state: &State = unsafe { &*node_state.cast() };
        todo!()
    }
}


pub fn init() -> &'static dyn api::Component {
    println!("Hello, world from xor");
    &Component {}
}
