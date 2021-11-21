use api::Mailbox;

struct Component;

struct Actor;

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, _arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        Box::new(Actor)
    }

    fn functions(&self) -> &'static [&'static str] {
        &["drop"]
    }

    fn name(&'static self) -> &'static str {
        "drop"
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let mut mailbox = mailbox.unwrap();
        runtime.spawn_task(async move {
            while mailbox.recv().await.is_some() {}
        });
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let mut mailbox = mailbox.unwrap();
        runtime.spawn_task(async move {
            while mailbox.recv().await.is_some() {}
        });
    }
}


pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
