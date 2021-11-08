use std::{collections::BTreeMap, error::Error};
use serde::Deserialize;

struct Spec;

impl api::ComponentSpec for Spec {
    fn create(&self, arguments: Vec<api::Argument>) -> api::Result<api::ActorFactory> {
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

        let key = &*Box::leak(Box::<[u8]>::from(config.key.as_bytes()));

        Ok(Box::new(move |mut runtime, meta| {
            Ok(Box::new(move || Box::pin(async move {
                let mut count = 0;
                let next = runtime.spawn(0, meta);

                while let Some(mut msg) = runtime.read().await {
                    for c in &mut msg[..] {
                        *c ^= key[count];
                        count = (count + 1) % key.len()
                    }
                    next.send(msg).await
                }

                Ok(())
            })))
        }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["xor"]
    }
}

pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
