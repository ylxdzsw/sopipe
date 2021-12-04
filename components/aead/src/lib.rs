use std::num::NonZeroU32;

use api::serde::Deserialize;
use ring::{aead::{Algorithm, BoundKey, UnboundKey, NonceSequence, Nonce, SealingKey, Aad, OpeningKey}, rand::{SecureRandom, SystemRandom}};

struct Component {
    rand: SystemRandom
}

#[derive(Clone, Copy)]
enum Role { Encoder, Decoder }

struct Counter {
    iv: [u8; 4],
    count: u64
}

impl Counter {
    fn new(iv: [u8; 4]) -> Self {
        Counter { iv, count: 0 }
    }
}

impl NonceSequence for Counter {
    fn advance(&mut self) -> Result<Nonce, ring::error::Unspecified> {
        let mut nonce = [0; 12];
        nonce[..4].copy_from_slice(&self.iv);
        nonce[4..].copy_from_slice(&self.count.to_be_bytes());
        self.count += 1;
        Ok(Nonce::assume_unique_for_key(nonce))
    }
}

struct Actor {
    key: Box<[u8]>,
    algo: &'static Algorithm,
    role: Role,
    rand: &'static SystemRandom
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            key: &'a str,

            #[serde(default)]
            algorithm: String,

            salt: Option<&'a str>,

            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("auth must have exactly 1 output")
        }

        let algo = match config.algorithm.to_lowercase().trim() {
            "" | "chacha20" | "chacha20_poly1305" => &ring::aead::CHACHA20_POLY1305,
            "aes_128_gcm" => &ring::aead::AES_128_GCM,
            "aes" | "aes_gcm" | "aes_256_gcm" => &ring::aead::AES_256_GCM,
            _ => panic!("unknown cypher")
        };
        let salt = config.salt.map(|x| x.as_bytes()).unwrap_or(b"sopipe_is_good");
        let key = derive_key(algo, salt, config.key.as_bytes());

        Box::new(Actor {
            key, algo,
            role: match config.function_name {
                "aead_encode" => Role::Encoder,
                "aead_decode" => Role::Decoder,
                _ => unreachable!()
            },
            rand: &self.rand
        })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["aead_encode", "aead_decode"]
    }

    fn name(&'static self) -> &'static str {
        "aead"
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        if let Some(stream_type) = metadata.get::<String>("stream_type") {
            if stream_type == "UDP" {
                eprintln!("WARNING: the aead module is not designed for UDP")
            }
        }

        let (forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, backward_mailbox) = runtime.channel();
        runtime.spawn_next(0, metadata, backward_address, forward_mailbox);
        match self.role {
            Role::Encoder => {
                runtime.spawn_task(self.encode(forward_address, mailbox.expect("no mailbox")));
                runtime.spawn_task(self.decode(address.expect("no address"), backward_mailbox));
            }
            Role::Decoder => {
                runtime.spawn_task(self.decode(forward_address, mailbox.expect("no mailbox")));
                runtime.spawn_task(self.encode(address.expect("no address"), backward_mailbox));
            }
        }
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        match self.role {
            Role::Encoder => runtime.spawn_task(self.encode(address.expect("no address"), mailbox.expect("no mailbox"))),
            Role::Decoder => runtime.spawn_task(self.decode(address.expect("no address"), mailbox.expect("no mailbox")))
        }
    }
}

impl Actor {
    async fn encode(&self, mut addr: impl api::Address, mut mail: impl api::Mailbox) {
        let mut iv = [0; 4];
        self.rand.fill(&mut iv).unwrap();

        let mut sealing_key = {
            let counter = Counter::new(iv);
            let unbound_key = UnboundKey::new(self.algo, &self.key).unwrap();
            SealingKey::new(unbound_key, counter)
        };

        if addr.send(Box::from(iv)).await.is_err() {
            return
        };

        // TODO: buffering?
        while let Some(mut msg) = mail.recv().await {
            // every packet is encrypted twice, one for the length and one for the content, without aad

            if msg.is_empty() {
                continue
            }

            let length_msg = msg.len() - 1;
            if length_msg > u16::MAX as _ {
                todo!()
            }

            let mut buf = Vec::with_capacity(2 + self.algo.tag_len() + msg.len() + self.algo.tag_len());
            buf.extend_from_slice(&u16::to_be_bytes(length_msg as _));
            sealing_key.seal_in_place_append_tag(Aad::empty(), &mut buf).unwrap();

            let tag = sealing_key.seal_in_place_separate_tag(Aad::empty(), &mut msg).unwrap();
            buf.extend_from_slice(&msg);
            buf.extend_from_slice(tag.as_ref());

            if addr.send(buf.into_boxed_slice()).await.is_err() {
                return
            }
        }
    }

    async fn decode(&self, mut addr: impl api::Address, mut mail: impl api::Mailbox) {
        let mut buf: Vec<u8> = vec![]; // todo: ring buffer (dequeue) for performance

        macro_rules! accumulate_buf_until_length {
            ($len: expr) => {{
                while buf.len() < $len {
                    if let Some(msg) = mail.recv().await {
                        buf.extend(&*msg)
                    } else {
                        return
                    }
                }
            }};
        }

        accumulate_buf_until_length!(4);
        let iv = buf[..4].try_into().unwrap();
        buf.drain(..4);

        let counter = Counter::new(iv);
        let unbound_key = UnboundKey::new(self.algo, &self.key).unwrap();
        let mut opening_key = OpeningKey::new(unbound_key, counter);

        loop {
            let length_msg_offset = 2 + self.algo.tag_len();
            accumulate_buf_until_length!(length_msg_offset);

            let length = match opening_key.open_in_place(Aad::empty(), &mut buf[..length_msg_offset]) {
                Ok(plain_text) => u16::from_be_bytes(plain_text[..2].try_into().unwrap()) as usize + 1,
                Err(_) => return eprintln!("aead: decryption failed")
            };

            let total_offset = length_msg_offset + length + self.algo.tag_len();
            accumulate_buf_until_length!(total_offset);

            let content = match opening_key.open_in_place(Aad::empty(), &mut buf[length_msg_offset..total_offset]) {
                Ok(plain_text) => plain_text.to_vec().into(),
                Err(_) => return eprintln!("aead: decryption failed")
            };

            if addr.send(content).await.is_err() {
                return
            };

            buf.drain(..total_offset);
        }
    }
}

fn derive_key(algo: &'static Algorithm, salt: &[u8], pass: &[u8]) -> Box<[u8]> {
    let mut key = vec![0; algo.key_len()];
    ring::pbkdf2::derive(ring::pbkdf2::PBKDF2_HMAC_SHA256, NonZeroU32::new(4096).unwrap(), salt, pass, &mut key);
    key.into_boxed_slice()
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    let rand = ring::rand::SystemRandom::new();
    Box::leak(Box::new(Component { rand }))
}
