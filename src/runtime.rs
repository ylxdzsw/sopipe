use std::collections::BTreeMap;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

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

    /// used by main.rs to spawn the initial source actors
    pub(crate) fn spawn(&'static self, node: &'static Node) -> JoinHandle<api::Result<()>> {
        let handler = RuntimeHandler { runtime: self, node, rx: None }.boxed();
        tokio::spawn((node.actor)(handler, Default::default()).unwrap())
    }
}

/// A handler for actors to call the runtime
struct RuntimeHandler {
    runtime: &'static Runtime,
    node: &'static Node,
    rx: Option<mpsc::Receiver<Box<[u8]>>>, // is there a way to construct a closed Receiver?
}

impl RuntimeHandler {
    pub(crate) fn spawn(&self, node: &'static Node, meta: BTreeMap<String, api::ArgumentValue>) -> Box<dyn api::Address> {
        let (tx, rx) = mpsc::channel(4);
        let handler = RuntimeHandler { runtime: self.runtime, node, rx: Some(rx) }.boxed();

        tokio::spawn((node.actor)(handler, meta).unwrap()); // The error is ignored. What to do here?
        Box::new(Address { tx })
    }
}

#[api::async_trait]
impl api::Runtime for RuntimeHandler {
    async fn read(&mut self) -> Option<Box<[u8]>> {
        self.rx.as_mut()?.recv().await
    }

    fn spawn(&self, index: usize, metadata: BTreeMap<String, api::ArgumentValue>) -> Box<dyn api::Address> {
        let node = &self.runtime.nodes[self.node.outputs[index]];
        RuntimeHandler::spawn(self, node, metadata)
    }

    fn spawn_conjugate(&self, metadata: BTreeMap<String, api::ArgumentValue>) -> Box<dyn api::Address> {
        let node = &self.runtime.nodes[self.node.conj];
        RuntimeHandler::spawn(self, node, metadata)
    }

    fn is_source(&self) -> bool {
        self.rx.is_none()
    }
}
