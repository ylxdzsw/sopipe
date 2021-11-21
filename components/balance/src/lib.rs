use std::sync::atomic::AtomicU64;

use api::serde::Deserialize;

struct Component;

struct Actor {
    count: AtomicU64, // the count don't warp. Instead, we calculate the modulo when using. Hopfully u64 won't deplete.
    n_outputs: usize
}

impl Actor {
    fn new(n_outputs: usize) -> Self {
        Self { count: 0.into(), n_outputs }
    }
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            method: Option<&'a str>,
            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        match &config.method {
            None | Some("round_robin") => Box::new(Actor::new(config.outputs.len())),
            _ => todo!()
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["balance"]
    }

    fn name(&'static self) -> &'static str {
        "balance"
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let next = self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let next = (next % self.n_outputs as u64) as usize;
        let (forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, backward_mailbox) = runtime.channel();
        runtime.spawn_next(next, metadata, backward_address, forward_mailbox);
        runtime.spawn_task(api::pass(address, Some(backward_mailbox)));
        runtime.spawn_task(api::pass(Some(forward_address), mailbox));
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
