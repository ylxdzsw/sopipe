
// time-based authentication. The client send the current time (must be unique) together with MAC code for verification

pub struct Client {
    key: &'static [u8]
}

impl Client {
    pub fn new(key: &'static [u8]) -> Self {
        Self { key }
    }
}

impl<R: api::Runtime> api::Actor<R> for Client {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
        // let (forward_address, forward_mailbox) = runtime.channel();
        // let (backward_address, backward_mailbox) = runtime.channel();
        // runtime.spawn_next(0, metadata, backward_address, forward_mailbox);
        // runtime.spawn_task(xor(self.key, forward_address, mailbox.expect("no mailbox")));
        // runtime.spawn_task(xor(self.key, address.expect("no address"), backward_mailbox));
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }
}


pub struct Server {
    key: &'static [u8]
}

impl Server {
    pub fn new(key: &'static [u8]) -> Self {
        Self { key }
    }
}

impl<R: api::Runtime> api::Actor<R> for Server {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }
}
