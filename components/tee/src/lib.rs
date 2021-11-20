use api::{Address, Mailbox};

struct Component;

struct Actor {
    n_outputs: usize
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let n_outputs = arguments.iter().find(|(name, _)| name == "outputs").unwrap().1.as_vec().unwrap().len();
        assert!(n_outputs >= 1);
        Box::new(Actor { n_outputs })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tee"]
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let mut addresses_next = Vec::with_capacity(self.n_outputs);
        let mut mailboxes_next = Vec::with_capacity(self.n_outputs);

        for i in 0..self.n_outputs {
            let (forward_address, forward_mailbox) = runtime.channel();
            let (backward_address, backward_mailbox) = runtime.channel();
            runtime.spawn_next(i, metadata.clone(), backward_address, forward_mailbox);
            addresses_next.push(forward_address);
            mailboxes_next.push(backward_mailbox);
        }

        let mut mailbox = mailbox.unwrap();
        runtime.spawn_task(async move {
            while let Some(msg) = mailbox.recv().await {
                let futures: Vec<_> = addresses_next.iter_mut().map(|addr| addr.send(msg.clone())).collect();
                for future in futures {
                    if future.await.is_err() {
                        // TODO: what to do? should we continue for the rest?
                        return
                    }
                }
            }
        });

        let address = address.unwrap();
        for mut mailbox in mailboxes_next.into_iter() {
            let mut address = address.clone();
            runtime.spawn_task(async move {
                while let Some(msg) = mailbox.recv().await {
                    if address.send(msg).await.is_err() {
                        return
                    }
                }
            })
        }
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
