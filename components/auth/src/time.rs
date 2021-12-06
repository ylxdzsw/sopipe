//! time-based authentication. The client send the current time (must be monotonic and unique) together with MAC code for verification

use std::sync::atomic::{AtomicU64, Ordering};

use api::{Address, Mailbox};

use crate::ALGORITHM;

pub struct Client {
    key: ring::hmac::Key
}

impl Client {
    pub fn new(key: ring::hmac::Key) -> Self {
        Self { key }
    }
}

impl<R: api::Runtime> api::Actor<R> for Client {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let (mut address_next, mailbox_next) = runtime.channel();
        runtime.spawn_next(0, metadata, address, mailbox_next);
        runtime.spawn_task(async move {
            let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as u64; // TODO: nanosecond is still in range of u64 (around 0.35 to the max). Use that instead?
            let mut msg = current_time.to_be_bytes().to_vec();
            let mac = ring::hmac::sign(&self.key, &msg);
            msg.extend_from_slice(mac.as_ref());
            if address_next.send(msg.into()).await.is_err() {
                return
            };

            api::pass(Some(address_next), mailbox).await;
        });
    }

    fn spawn_composite(&'static self, _runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }
}

// Problem: if some users who have the key purposely adjust their clocks to be slightly faster and keep making connections, they can block other users because this would sets LAST_TIME to be earlier than actual clock.
static LAST_TIME: AtomicU64 = AtomicU64::new(0); // this is shared across all streams because they all share the same key.

pub struct Server {
    key: ring::hmac::Key
}

impl Server {
    pub fn new(key: ring::hmac::Key) -> Self {
        Self { key }
    }
}

impl<R: api::Runtime> api::Actor<R> for Server {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let mut mailbox = mailbox.unwrap();

        runtime.spawn_task_with_runtime(move |runtime| async move {
            let mut buf = vec![];
            let header_len = 8 + ALGORITHM.digest_algorithm().output_len; // timestamp (u64) + mac

            while let Some(msg) = mailbox.recv().await {
                buf.extend_from_slice(&msg);
                if buf.len() >= header_len {
                    break
                }
            }

            // 1. verify timestamp
            let time_stamp = u64::from_be_bytes(buf[..8].try_into().unwrap());
            let current_time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros() as u64;
            if time_stamp < current_time - 5_000_000 || time_stamp > current_time + 1_000_000 { // older than 5s or earlier than 1s
                return
            }

            let last_time = LAST_TIME.load(Ordering::Relaxed);
            if time_stamp <= last_time {
                return
            }

            loop {
                match LAST_TIME.compare_exchange_weak(last_time, time_stamp, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(last_time) if last_time < time_stamp => continue, // LAST_TIME has been updated, but it is still OK as long as time_stamp is still larger
                    Err(_) => return
                }
            }

            // 2. verify MAC
            if ring::hmac::verify(&self.key, &buf[..8], &buf[8..header_len]).is_err() {
                if let Some(origin) = metadata.get::<std::net::SocketAddr>("origin_addr") {
                    eprintln!("auth: failed attempt from {}", origin)
                } else {
                    eprintln!("auth: failed attempt")
                }
                return // TODO: cut connection
            }

            // 3. forwarding
            let (mut address_next, mailbox_next) = runtime.channel();
            runtime.spawn_next(0, metadata, address, mailbox_next);

            #[allow(clippy::collapsible_if)]
            if buf.len() > header_len {
                if address_next.send(Box::from(&buf[header_len..])).await.is_err() {
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

