use oh_my_rust::*;
use std::{collections::BTreeMap, error::Error};
use thiserror::Error;

struct Spec {

}


struct Actor {

}

struct Component {
    key: Option<Box<[u8]>>
}

impl api::Component for Component {
    fn spawn(&self, runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

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
    async fn run(self: Box<Self>, ) -> Result<(), Box<dyn Error + Send + Sync>> {
        todo!()
    }
}

impl api::ComponentSpec for Spec {
    fn create(&self, arguments: Vec<api::Argument>) -> Result<Box<dyn api::Component>, Box<dyn Error + Send + Sync>> {
        let mut key = None;
        task!("creating instance of xor");
        for arg in arguments.into_iter() {
            let api::Argument(name, value) = arg;
            info!("{}: {:?}", name, value);
            if name.is_empty() || name == "key" {
                if key.is_none() {
                    if let api::ArgumentValue::String(x) = value {
                        key = Some(x.into_bytes().into_boxed_slice())
                    } else {
                        return Err(Box::new(XorError::InvalidArgument("key should be string")))
                    }
                } else {
                    return Err(Box::new(XorError::InvalidArgument("key set twice")))
                }
            }
        }
        Ok(Box::new(Component { key }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["xor"]
    }

}


pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
