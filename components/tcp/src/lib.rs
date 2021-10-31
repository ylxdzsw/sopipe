use std::{error::Error, ptr::NonNull};

struct Actor {

}

struct Component {

}

#[api::async_trait]
impl api::Actor for Actor {
    async fn feed(&mut self, ) {}
}

impl api::Component for Component {
    fn create(&self,) -> Result<NonNull<()>, Box<dyn Error>> {

        Err("fuck".into())
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

pub fn init() -> &'static dyn api::Component {
    println!("Hello, world from tcp");
    &Component {}
}
