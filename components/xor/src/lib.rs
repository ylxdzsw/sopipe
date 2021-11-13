use api::serde::Deserialize;

struct Component;

struct Actor {
    key: &'static [u8]
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            key: &'a str,
            outputs: Vec<String>,
            function_name: &'a str,
            #[serde(default)]
            read_only: bool,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("xor must have exactly 1 output")
        }

        let key = &*Box::leak(Box::<[u8]>::from(config.key.as_bytes()));

        Box::new(Actor { key })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["xor"]
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let (forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, backward_mailbox) = runtime.channel();
        runtime.spawn_next(0, metadata, backward_address, forward_mailbox);
        runtime.spawn_task(xor(self.key, forward_address, mailbox.expect("no mailbox")));
        runtime.spawn_task(xor(self.key, address.expect("no address"), backward_mailbox));
    }

    fn spawn_composite(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_source(&'static self, runtime: R) {
        todo!()
    }
}

async fn xor(key: &[u8], mut addr: impl api::Address, mut mail: impl api::Mailbox) {
    let mut count = 0;

    while let Some(mut msg) = mail.recv().await {
        for c in &mut msg[..] {
            *c ^= key[count];
            count = (count + 1) % key.len()
        }
        if addr.send(msg).await.is_err() {
            break
        }
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
