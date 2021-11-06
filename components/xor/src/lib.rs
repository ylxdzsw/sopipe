use oh_my_rust::*;
use std::{collections::BTreeMap, error::Error};
use anyhow::anyhow;
use serde::Deserialize;

struct Spec;

struct Actor {
    key: Box<[u8]>
}

struct Component {
    key: Option<Box<[u8]>>
}

impl api::ComponentSpec for Spec {
    fn create(&self, arguments: Vec<api::Argument>) -> Result<Box<dyn api::Component>, Box<dyn Error + Send + Sync>> {
        #[derive(Debug, Deserialize)]
        struct Config<'a> {
            key: &'a str,
            direction: &'a str,
            outputs: Vec<String>,
            function_name: &'a str,
            #[serde(default)]
            read_only: bool,
        }

        let config: Config = api::helper::parse_args(&arguments).unwrap();
        println!("{:?}", config);

        unimplemented!();
        // Ok(Box::new(Component { key }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["xor"]
    }

}

impl api::Component for Component {
    fn spawn(&self, runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

}




#[api::async_trait]
impl api::Actor for Actor {
    async fn run(self: Box<Self>, ) -> Result<(), Box<dyn Error + Send + Sync>> {
        todo!()
    }
}




pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
