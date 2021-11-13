use super::Node;

pub struct Runtime {
    nodes: &'static [Node],
    runlevel: tokio::sync::watch::Receiver<api::RunLevel>
}

impl Runtime {
    pub(crate) fn new(nodes: &'static [Node], runlevel: tokio::sync::watch::Receiver<api::RunLevel>) -> Self {
        Self { nodes, runlevel }
    }

    pub(crate) fn spawn_source(&'static self, node: &'static Node) {
        let handler = Box::new(RuntimeHandler { runtime: self, node, is_composite: false });
        node.forward_actor.spawn_source(handler)
    }
}

#[derive(Clone)]
pub struct Address(tokio::sync::mpsc::Sender<Box<[u8]>>);

impl api::Address for Address {
    fn send(&mut self, msg: Box<[u8]>) -> std::pin::Pin<Box<dyn std::future::Future<Output=Result<(), ()>> + Send + '_>> {
        Box::pin(async { self.0.send(msg).await.map_err(|_| ()) })
    }
}

pub struct Mailbox(tokio::sync::mpsc::Receiver<Box<[u8]>>);

impl api::Mailbox for Mailbox {
    #[allow(clippy::type_complexity)]
    fn recv(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output=Option<Box<[u8]>>> + Send + '_>> {
        Box::pin(self.0.recv())
    }
}

/// A handler for actors to call the runtime
pub struct RuntimeHandler {
    runtime: &'static Runtime,
    node: &'static Node,
    is_composite: bool // composite nodes are not allowed to spawn next
}

impl api::Runtime for RuntimeHandler {
    type Address = Address;
    type Mailbox = Mailbox;

    fn spawn_next(&self, index: usize, metadata: api::MetaData, address: impl Into<Option<Self::Address>>, mailbox: impl Into<Option<Self::Mailbox>>) {
        if self.is_composite {
            panic!("cannot spawn in composite components")
        }

        let address = address.into();
        let mailbox = mailbox.into();

        let next = &self.runtime.nodes[self.node.outputs[index]];

        #[allow(clippy::ptr_eq)]
        if next.forward_actor as *const _ as *const u8 == next.backward_actor as *const _ as *const u8 {
            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: false });
            next.forward_actor.spawn(handler, metadata, address, mailbox)
        } else {
            let (forward_address_next, forward_mailbox_next) = self.channel();
            let (backward_address_next, backward_mailbox_next) = self.channel();

            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: true });
            next.forward_actor.spawn_composite(handler, metadata.clone(), Some(forward_address_next), mailbox);

            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: true });
            next.backward_actor.spawn_composite(handler, metadata.clone(), address, Some(backward_mailbox_next));

            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: false });
            handler.spawn_next(0, metadata, Some(backward_address_next), Some(forward_mailbox_next))
        }
    }

    fn channel(&self) -> (Self::Address, Self::Mailbox) {
        // TODO: componenets give hints about buffer size, so that fast components (like xor) don't increase the overal buffer in the stack
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        (Address(tx), Mailbox(rx))
    }

    fn spawn_task<F: std::future::Future + Send + 'static>(&self, task: F) where F::Output: Send {
        tokio::spawn(async {
            self.node.task_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            task.await;
            self.node.task_count.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        });
    }

    fn get_runlevel(&self) -> api::RunLevel {
        *self.runtime.runlevel.borrow()
    }

    fn watch_runlevel(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output=()> + Send + '_>> {
        Box::pin(async { self.runtime.runlevel.clone().changed().await.unwrap(); })
    }
}
