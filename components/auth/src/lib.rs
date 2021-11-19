use std::num::NonZeroU32;

use api::serde::Deserialize;

struct Component;

static ALGORITHM: ring::hmac::Algorithm = ring::hmac::HMAC_SHA256;

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

            salt: Option<&'a str>,

            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("auth must have exactly 1 output")
        }

        let salt = config.salt.map(|x| x.as_bytes()).unwrap_or(b"sopipe_is_good");
        let key = derive_key(salt, config.key.as_bytes());

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

fn derive_key(salt: &[u8], pass: &[u8]) -> ring::hmac::Key {
    let mut key = vec![0; ALGORITHM.digest_algorithm().output_len];
    ring::pbkdf2::derive(ring::pbkdf2::PBKDF2_HMAC_SHA256, NonZeroU32::new(4096).unwrap(), salt, pass, &mut key);
    ring::hmac::Key::new(ALGORITHM, &key)
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
