use api::serde::Deserialize;
use miniz_oxide::{deflate::{core::CompressorOxide, stream::deflate}, inflate::stream::{InflateState, inflate}, StreamResult, DataFormat, MZFlush};

struct Component;

#[derive(Clone, Copy)]
enum Role { Encoder, Decoder }

struct Actor {
    level: u8,
    role: Role
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            level: Option<u8>,

            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("miniz must have exactly 1 output")
        }

        Box::new(Actor {
            level: config.level.unwrap_or(1),
            role: match config.function_name {
                "deflate" => Role::Encoder,
                "inflate" => Role::Decoder,
                _ => unreachable!()
            }
        })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["deflate", "inflate"]
    }

    fn name(&'static self) -> &'static str {
        "miniz"
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
                runtime.spawn_task(self.deflate(forward_address, mailbox.expect("no mailbox")));
                runtime.spawn_task(self.inflate(address.expect("no address"), backward_mailbox));
            }
            Role::Decoder => {
                runtime.spawn_task(self.inflate(forward_address, mailbox.expect("no mailbox")));
                runtime.spawn_task(self.deflate(address.expect("no address"), backward_mailbox));
            }
        }
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        match self.role {
            Role::Encoder => runtime.spawn_task(self.deflate(address.expect("no address"), mailbox.expect("no mailbox"))),
            Role::Decoder => runtime.spawn_task(self.inflate(address.expect("no address"), mailbox.expect("no mailbox")))
        }
    }
}

impl Actor {
    async fn deflate(&self, mut addr: impl api::Address, mut mail: impl api::Mailbox) {
        let mut compressor = CompressorOxide::default();
        compressor.set_format_and_level(DataFormat::Raw, self.level);

        let mut buffer = vec![0; 65536];
        while let Some(msg) = mail.recv().await {
            if buffer.len() < msg.len() * 2 { // assume that compress data is not twice longer. Need to revisit this later.
                buffer.resize(msg.len() * 2, 0)
            }

            let StreamResult { bytes_consumed, bytes_written, status } = deflate(&mut compressor, &msg, &mut buffer, MZFlush::Sync);
            assert!(status.is_ok());
            assert_eq!(bytes_consumed, msg.len());

            if addr.send(buffer[..bytes_written].to_vec().into()).await.is_err() {
                return
            }
        }
    }

    async fn inflate(&self, mut addr: impl api::Address, mut mail: impl api::Mailbox) {
        let mut decompressor = InflateState::new_boxed(DataFormat::Raw);

        let mut buffer = vec![0; 65536];
        while let Some(msg) = mail.recv().await {
            let mut offset = 0;

            loop {
                let StreamResult { bytes_consumed, bytes_written, status: _ } = inflate(&mut decompressor, &msg[offset..], &mut buffer, MZFlush::Sync);

                if addr.send(buffer[..bytes_written].to_vec().into()).await.is_err() {
                    return
                }

                offset += bytes_consumed;

                if offset >= msg.len() {
                    break;
                }
            }
        }
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}

// Note for checking correctness: compress it and verify the result using this command:
// `printf "\x1f\x8b\x08\x00\x00\x00\x00\x00\00\00" | cat - q | gzip -dc`
// from https://unix.stackexchange.com/questions/22834/how-to-uncompress-zlib-data-in-unix
