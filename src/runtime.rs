use std::{future::Future, pin::Pin, sync::atomic::AtomicU8};

use super::{Counter, Node};

pub struct Runtime {
    nodes: &'static [Node],
    runlevel: AtomicU8
}

impl Runtime {
    pub(crate) fn new(nodes: &'static [Node]) -> Self {
        Self { nodes, runlevel: (api::RunLevel::Init as u8).into() }
    }

    pub(crate) fn spawn_source(&'static self, node: &'static Node) {
        let handler = RuntimeHandler { runtime: self, node, is_composite: false };
        node.forward_actor.spawn_source(handler)
    }

    pub(crate) fn set_run_level(&'static self, runlevel: api::RunLevel) {
        self.runlevel.store(runlevel as _, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct Address(tokio::sync::mpsc::Sender<Box<[u8]>>);

impl api::Address for Address {
    fn send(&mut self, msg: Box<[u8]>) -> Pin<Box<dyn Future<Output=Result<(), ()>> + Send + '_>> {
        Box::pin(async { self.0.send(msg).await.map_err(|_| ()) })
    }
}

pub struct Mailbox(tokio::sync::mpsc::Receiver<Box<[u8]>>);

impl api::Mailbox for Mailbox {
    #[allow(clippy::type_complexity)]
    fn recv(&mut self) -> Pin<Box<dyn Future<Output=Option<Box<[u8]>>> + Send + '_>> {
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
            let handler = RuntimeHandler { runtime: self.runtime, node: next, is_composite: false };
            next.forward_actor.spawn(handler, metadata, address, mailbox)
        } else {
            let (forward_address_next, forward_mailbox_next) = self.channel();
            let (backward_address_next, backward_mailbox_next) = self.channel();

            let handler = RuntimeHandler { runtime: self.runtime, node: next, is_composite: true };
            next.forward_actor.spawn_composite(handler, metadata.clone(), Some(forward_address_next), mailbox);

            let handler = RuntimeHandler { runtime: self.runtime, node: next, is_composite: true };
            next.backward_actor.spawn_composite(handler, metadata.clone(), address, Some(backward_mailbox_next));

            let handler = RuntimeHandler { runtime: self.runtime, node: next, is_composite: false };
            handler.spawn_next(0, metadata, Some(backward_address_next), Some(forward_mailbox_next))
        }
    }

    fn channel(&self) -> (Self::Address, Self::Mailbox) {
        // TODO: componenets give hints about buffer size, so that fast components don't increase the overal buffer in the pipeline
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        (Address(tx), Mailbox(rx))
    }

    fn spawn_task<F: Future + Send + 'static>(&self, task: F) where F::Output: Send {
        tokio::spawn(async {
            let _c = Counter::new(&self.node.task_count); // use Drop in case of panic
            task.await;
        });
    }

    fn spawn_task_with_runtime<C, F>(&self, task: C) where
        C: FnOnce(Self) -> F,
        F: Future + Send + 'static,
        F::Output: Send
    {
        let handler = RuntimeHandler { ..*self };
        self.spawn_task(task(handler))
    }

    fn get_runlevel(&self) -> api::RunLevel {
        const INIT: u8 = api::RunLevel::Init as _;
        const RUN: u8 = api::RunLevel::Run as _;
        const SHUT: u8 = api::RunLevel::Shut as _;

        match self.runtime.runlevel.load(std::sync::atomic::Ordering::Relaxed) {
            INIT => api::RunLevel::Init,
            RUN => api::RunLevel::Run,
            SHUT => api::RunLevel::Shut,
            _ => unreachable!()
        }
    }
}
