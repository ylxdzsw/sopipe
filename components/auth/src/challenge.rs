use api::{Address, Mailbox};
use ring::rand::SecureRandom;

// challenge-based authentication. The server send a random nounce (challenge) and the client must reply with the coresponding MAC code.

use crate::ALGORITHM;

const NOUNCE_LEN: usize = 20;

pub struct Client {
    key: ring::hmac::Key,
}

impl Client {
    pub fn new(key: ring::hmac::Key) -> Self {
        Self { key }
    }
}

impl<R: api::Runtime> api::Actor<R> for Client {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let (mut forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, mut backward_mailbox) = runtime.channel();
        runtime.spawn_next(0, metadata, backward_address, forward_mailbox);

        runtime.spawn_task_with_runtime(|runtime| async move {
            let mut buf = vec![];

            while buf.len() < NOUNCE_LEN {
                if let Some(msg) = backward_mailbox.recv().await {
                    buf.extend_from_slice(&msg);
                } else {
                    return
                }
            }

            if buf.len() != NOUNCE_LEN {
                eprintln!("protocol error");
                return
            }

            let mac = ring::hmac::sign(&self.key, &buf);
            if forward_address.send(Box::from(mac.as_ref())).await.is_err() {
                return
            }

            runtime.spawn_task(api::pass(address, Some(backward_mailbox)));
            runtime.spawn_task(api::pass(Some(forward_address), mailbox));
        });
    }

    fn spawn_composite(&'static self, _runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }
}

pub struct Server {
    key: ring::hmac::Key,
    rand: ring::rand::SystemRandom
}

impl Server {
    pub fn new(key: ring::hmac::Key) -> Self {
        Self { key, rand: ring::rand::SystemRandom::new() }
    }
}

impl<R: api::Runtime> api::Actor<R> for Server {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let mut mailbox = mailbox.unwrap();
        let mut address = address.unwrap();

        runtime.spawn_task_with_runtime(|runtime| async move {
            let mut nounce = [0; NOUNCE_LEN];
            self.rand.fill(&mut nounce).unwrap();
            if address.send(Box::new(nounce)).await.is_err() {
                return
            }

            let mut buf = vec![];

            while buf.len() < ALGORITHM.digest_algorithm().output_len {
                if let Some(msg) = mailbox.recv().await {
                    buf.extend_from_slice(&msg);
                } else {
                    return
                }
            }

            let (mac, rest) = buf.split_at(ALGORITHM.digest_algorithm().output_len);
            if ring::hmac::verify(&self.key, &nounce, mac).is_err() {
                if let Some(origin) = metadata.get::<std::net::SocketAddr>("origin_addr") {
                    eprintln!("auth: failed attempt from {}", origin)
                } else {
                    eprintln!("auth: failed attempt")
                }
                return // TODO: cut connection
            }

            let (mut address_next, mailbox_next) = runtime.channel();
            runtime.spawn_next(0, metadata, address, mailbox_next);

            #[allow(clippy::collapsible_if)]
            if !rest.is_empty() {
                if address_next.send(Box::from(rest)).await.is_err() {
                    return
                }
            }

            api::pass(Some(address_next), Some(mailbox)).await
        });
    }

    fn spawn_composite(&'static self, _runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }
}
