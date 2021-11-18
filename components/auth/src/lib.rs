use api::serde::Deserialize;

struct Component;

mod time;
mod challenge;

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            key: &'a str,

            #[serde(default)]
            method: String,
            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("auth must have exactly 1 output")
        }

        let key = &*Box::leak(Box::<[u8]>::from(config.key.as_bytes()));

        match &config.method[..] {
            "" | "time" => match config.function_name {
                "auth_client" => Box::new(time::Client::new(key)),
                "auth_server" => Box::new(time::Server::new(key)),
                _ => unreachable!()
            }
            "challenge" => match config.function_name {
                "auth_client" => Box::new(challenge::Client::new(key)),
                "auth_server" => Box::new(challenge::Server::new(key)),
                _ => unreachable!()
            },
            _ => panic!("unkown auth method. Avaliable: time, challenge")
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["auth_client", "auth_server"]
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
