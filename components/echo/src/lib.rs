struct Component;

struct Actor;

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        assert!(arguments.iter().find(|(name, _)| name == "outputs").unwrap().1.as_vec().unwrap().is_empty());
        Box::new(Actor)
    }

    fn functions(&self) -> &'static [&'static str] {
        &["echo"]
    }

    fn name(&'static self) -> &'static str {
        "echo"
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        runtime.spawn_task(api::pass(address, mailbox));
    }
}


pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
