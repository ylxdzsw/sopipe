use api::serde::Deserialize;

mod server;

struct Component;

#[derive(Debug, Deserialize)]
#[serde(crate="api::serde")]
struct Config<'a> {
    outputs: Vec<&'a str>,
    function_name: &'a str,
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let config: Config = api::parse_args(&arguments).unwrap();

        match config.function_name {
            "socks5_server" => {
                assert!(config.outputs.len() == 1);
                Box::new(server::Actor)
            }
            "socks5_client" => {
                todo!()
            }
            _ => unreachable!()
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["socks5_server", "socks5_client"]
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}

