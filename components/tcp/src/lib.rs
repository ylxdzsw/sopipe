use std::{error::Error, ptr::NonNull};
use api::Argument;
use thiserror::Error;

struct Actor {

}

struct Component {

}

enum State { Forward(u16), Backward }

#[derive(Error, Debug)]
pub enum TcpError {
    #[error("Invalid arguments. Detail: {0}")]
    InvalidArgument(&'static str),
}

#[api::async_trait]
impl api::Actor for Actor {
    async fn feed(&mut self, ) {}
}

impl api::Component for Component {
    fn create(&self, arguments: Vec<Argument>) -> Result<NonNull<()>, Box<dyn Error + Send + Sync>> {
        let state;
        match &arguments.iter().find(|x| x.0 == "direction").unwrap().1.as_string().unwrap()[..] {
            "forward" => {
                let port = arguments.iter().find(|x| x.0.is_empty() || x.0 == "port").unwrap().1.as_int().unwrap();
                state = State::Forward(*port as _)
            },
            "backward" => {
                state = State::Backward
            }
            _ => unreachable!()
        }

        Ok(NonNull::from(Box::leak(Box::new(state))).cast())
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }

    fn spawn(&self, node_state: *const ()) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        todo!()
    }
}

pub fn init() -> &'static dyn api::Component {
    println!("Hello, world from tcp");
    &Component {}
}
