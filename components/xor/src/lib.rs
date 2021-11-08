use std::{collections::BTreeMap, error::Error};
use serde::Deserialize;

struct Spec;

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
    fn create(&'static self, mut runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> api::Result<api::Actor> {
        Ok(Box::new(move || Box::pin(async move {
            let mut count = 0;
            let next = runtime.spawn(0, args);

            while let Some(mut msg) = runtime.read().await {
                for c in &mut msg[..] {
                    *c ^= self.key[count];
                    count = (count + 1) % self.key.len()
                }
                next.send(msg).await
            }

            Ok(())
        })))
    }
}

pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
