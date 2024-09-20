use api::serde::Deserialize;

struct Component;

mod client;

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            user_id: &'a str,

            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("auth must have exactly 1 output")
        }

        let user_id = parse_uid(config.user_id).unwrap();

        match config.function_name {
            "vmess_client" => Box::new(client::Client::new(user_id)),
            _ => unreachable!()
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["vmess_client"]
    }

    fn name(&'static self) -> &'static str {
        "vmess"
    }
}

fn parse_uid(x: &str) -> Option<[u8; 16]> {
    let x = x.replace('-', "");
    let list: Vec<_> = (0..32).step_by(2).map(|i| u8::from_str_radix(&x[i..i+2], 16).unwrap()).collect();
    list.get(0..16).and_then(|x| x.try_into().ok())
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
