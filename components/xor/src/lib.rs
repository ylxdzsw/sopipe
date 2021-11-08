use std::{collections::BTreeMap, error::Error};
use serde::Deserialize;

struct Spec;

struct Actor {
    runtime: Box<dyn api::Runtime>,
    next: Box<dyn api::Address>,
    key: Box<[u8]>,
    count: usize
}

struct Component {
    key: Box<[u8]>
}

impl api::ComponentSpec for Spec {
    fn create(&self, arguments: Vec<api::Argument>) -> api::Result<Box<dyn api::Component>> {
        #[allow(dead_code)]
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

        Ok(Box::new(Component { key: Box::from(config.key.as_bytes()) }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["xor"]
    }
}

impl api::Component for Component {
    fn spawn(&self, runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> api::Result<Box<dyn api::Actor>> {
        let next = runtime.spawn(0, args);
        Ok(Box::new(Actor { runtime, next, key: self.key.clone(), count: 0 }))
    }
}

#[api::async_trait]
impl api::Actor for Actor {
    async fn run(mut self: Box<Self>) -> api::Result<()> {
        while let Some(mut msg) = self.runtime.read().await {
            for c in &mut msg[..] {
                *c ^= self.key[self.count];
                self.count = (self.count + 1) % self.key.len()
            }
            self.next.send(msg).await
        }

        Ok(())
    }
}

pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
