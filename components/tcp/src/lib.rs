use std::{error::Error, ptr::NonNull};

use api::Argument;

struct Actor {

}

struct Component {

}

#[api::async_trait]
impl api::Actor for Actor {
    async fn feed(&mut self, ) {}
}

impl api::Component for Component {
    fn create(&self, arguments: Vec<Argument>) -> Result<NonNull<()>, Box<dyn Error + Send + Sync>> {

        Err("fuck".into())
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
