use std::collections::BTreeMap;

use tokio::sync::mpsc;

use super::Node;
use super::ExtForBoxed;

struct Address {
    tx: mpsc::Sender<Box<[u8]>>
}

#[api::async_trait]
impl api::Address for Address {
    async fn send(&self, msg: Box<[u8]>) {
        self.tx.send(msg).await.unwrap()
    }
}

pub struct Runtime {
    nodes: &'static [Node]
}

impl Runtime {
    pub(crate) fn new(nodes: &'static [Node]) -> Self {
        Self { nodes }
    }

    pub(crate) fn spawn(&'static self, node: &'static Node) -> Box<dyn api::Actor> {
        let handler = RuntimeHandler { runtime: self, node, rx: None }.boxed();
        node.comp.spawn(handler, Default::default()).unwrap()
    }
}

/// A handler for actors to call the runtime
struct RuntimeHandler {
    runtime: &'static Runtime,
    node: &'static Node,
    rx: Option<mpsc::Receiver<Box<[u8]>>> // is there a way to construct a closed Receiver?
}

#[api::async_trait]
impl api::Runtime for RuntimeHandler {
    async fn read(&mut self) -> Option<Box<[u8]>> {
        self.rx.as_mut()?.recv().await
    }

    fn spawn(&self, index: usize, args: BTreeMap<String, api::ArgumentValue>) -> Box<dyn api::Address> {
        let next_node = &self.runtime.nodes[self.node.outputs[index]];
        let (tx, rx) = mpsc::channel(4);
        let handler = RuntimeHandler { runtime: self.runtime, node: next_node, rx: Some(rx) };

        let mut actor = self.node.comp.spawn(Box::new(handler), args).unwrap();
        tokio::spawn(async move {
            actor.run().await;
        });
        Box::new(Address { tx })
    }

    fn spawn_conjugate(&self, args: BTreeMap<String, api::ArgumentValue>) -> Box<dyn api::Address> {
        todo!()
    }
}
