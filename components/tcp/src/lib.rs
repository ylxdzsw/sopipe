use std::sync::atomic::AtomicU64;
use api::serde::Deserialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener};

struct Component;

mod listen;
mod active;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(crate="api::serde")]
struct Config<'a> {
    port: Option<u16>,

    addr: Option<&'a str>,

    #[serde(default)]
    active: bool,

    outputs: Vec<&'a str>,
    function_name: String,
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let config: Config = api::parse_args(&arguments).unwrap();

        if config.active {
            todo!()
        } else {
            Box::new(listen::Actor::new(config))
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
