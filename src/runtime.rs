use api::tokio::sync::mpsc;

use super::Node;

pub struct Runtime {
    nodes: &'static [Node]
}

impl Runtime {
    pub(crate) fn new(nodes: &'static [Node]) -> Self {
        Self { nodes }
    }

    pub(crate) fn spawn(&'static self, node: &'static Node) {
        let handler = Box::new(RuntimeHandler { runtime: self, node, is_composite: false });
        node.forward_actor.spawn_source(handler)
    }
}

/// A handler for actors to call the runtime
struct RuntimeHandler {
    runtime: &'static Runtime,
    node: &'static Node,
    is_composite: bool, // composite nodes are not allowed to spawn next
}

impl api::Runtime for RuntimeHandler {
    fn spawn_next(&self, index: usize, metadata: api::MetaData, address: Option<api::Address>, mailbox: Option<api::Mailbox>) {
        if self.is_composite {
            panic!("cannot spawn in composite components")
        }

        let next = &self.runtime.nodes[index];

        #[allow(clippy::ptr_eq)]
        if next.forward_actor as *const _ as *const u8 == next.backward_actor as *const _ as *const u8 {
            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: false });
            next.forward_actor.spawn(handler, metadata, address, mailbox)
        } else {
            let (forward_address_next, forward_mailbox_next) = mpsc::channel(4);
            let (backward_address_next, backward_mailbox_next) = mpsc::channel(4);

            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: true });
            next.forward_actor.spawn_composite(handler, metadata.clone(), Some(forward_address_next), mailbox);

            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: true });
            next.backward_actor.spawn_composite(handler, metadata.clone(), address, Some(backward_mailbox_next));

            let handler = Box::new(RuntimeHandler { runtime: self.runtime, node: next, is_composite: false });
            handler.spawn_next(0, metadata, Some(backward_address_next), Some(forward_mailbox_next))
        }
    }
}
