// challenge-based authentication. The server send a random nounce (challenge) and the client must reply with the coresponding MAC code.

pub struct Client {
    key: &'static [u8]
}

impl Client {
    pub fn new(key: ring::hmac::Key) -> Self {
        todo!()
    }
}

impl<R: api::Runtime> api::Actor<R> for Client {
    fn spawn(&'static self, runtime: R, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }
}


pub struct Server {
    key: &'static [u8]
}

impl Server {
    pub fn new(key: ring::hmac::Key) -> Self {
        todo!()
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
