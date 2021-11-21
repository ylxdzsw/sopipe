use std::time::Duration;

use api::serde::Deserialize;

struct Component;

struct Actor {
    drop_rate: Option<f64>,
    interval: Duration,
    budget: Budget
}

#[derive(Clone)]
struct Budget {
    size: Option<i64>, // bytes
    n_packets: Option<i64>,
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        // TODO: allow numbers with suffix to indicate the unit in the script?

        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            size: Option<i64>, // bytes
            n_packets: Option<i64>,
            drop_rate: Option<u64>, // percentage (0-100)

            interval: Option<u64>, // ms

            outputs: Vec<&'a str>,
            function_name: &'a str,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        if config.outputs.len() != 1 {
            panic!("throttle must have exactly 1 output")
        }

        let budget = Budget {
            size: config.size,
            n_packets: config.n_packets
        };

        Box::new(Actor {
            drop_rate: config.drop_rate.map(|x| x as f64 / 100.),
            interval: config.interval.map(Duration::from_millis)
                .unwrap_or_else(|| Duration::from_secs(1)), // defaults to one second
            budget,
        })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["throttle"]
    }

    fn name(&'static self) -> &'static str {
        "throttle"
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let (forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, backward_mailbox) = runtime.channel();
        runtime.spawn_next(0, metadata, backward_address, forward_mailbox);
        runtime.spawn_task(self.throttle(forward_address, mailbox.expect("no mailbox")));
        runtime.spawn_task(self.throttle(address.expect("no address"), backward_mailbox));
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        runtime.spawn_task(self.throttle(address.expect("no address"), mailbox.expect("no mailbox")));
    }
}

impl Actor {
    async fn throttle(&self, mut addr: impl api::Address, mut mail: impl api::Mailbox) {
        let mut budget = self.budget.clone();
        let mut last_tick = tokio::time::Instant::now();

        while let Some(msg) = mail.recv().await {
            if last_tick.elapsed() >= self.interval {
                budget = self.budget.clone();
                last_tick = tokio::time::Instant::now();
            }

            if let Some(drop_rate) = self.drop_rate {
                if rand::random::<f64>() < drop_rate {
                    eprintln!("dropped one");
                    continue
                }
            }

            if let Some(n_packets) = &mut budget.n_packets {
                *n_packets -= 1;
                if *n_packets < 0 { // budget used up, wait for next interval
                    tokio::time::sleep_until(last_tick + self.interval).await;
                    budget = self.budget.clone();
                    last_tick = tokio::time::Instant::now();
                }
            }

            if let Some(size) = &mut budget.size {
                *size -= msg.len() as i64;
                if *size < 0 { // budget used up, wait for next interval
                    tokio::time::sleep_until(last_tick + self.interval).await;
                    budget = self.budget.clone();
                    last_tick = tokio::time::Instant::now();
                }
            }

            if addr.send(msg).await.is_err() {
                return
            }
        }
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
