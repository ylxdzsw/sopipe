use std::{collections::BTreeSet, sync::atomic::AtomicU32};

mod script;
mod runtime;

type R = runtime::RuntimeHandler;

/// An (composite) actor with runtime-tracked states
struct Node {
    forward_actor: &'static dyn api::Actor<R>,
    backward_actor: &'static dyn api::Actor<R>,
    outputs: &'static [usize],

    task_count: AtomicU32
}

impl Node {
    fn new(forward_actor: &'static dyn api::Actor<R>, backward_actor: &'static dyn api::Actor<R>, outputs: &'static [usize]) -> Self {
        Self { forward_actor, backward_actor, outputs, task_count: Default::default() }
    }
}

#[allow(clippy::vec_init_then_push)]
fn main() {
    let mut components = vec![];

    #[cfg(feature="stdio")]
    components.push(stdio::init());

    #[cfg(feature="xor")]
    components.push(xor::init());

    #[cfg(feature="tcp")]
    components.push(tcp::init());

    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: sopipe <script>");
        std::process::exit(0);
    }

    let nodes: &_ = script::load_script(&args[1], &components).leak();

    let runtime = Box::leak(Box::new(runtime::Runtime::new(nodes)));

    let tokio_rt = tokio::runtime::Runtime::new().unwrap();

    tokio_rt.block_on(async move {
        let non_source: BTreeSet<_> = nodes.iter()
            .flat_map(|x| x.outputs.iter())
            .copied().collect();
        for (i, x) in nodes.iter().enumerate() {
            if non_source.contains(&i) {
                continue
            }
            assert_eq!(x.forward_actor as *const _ as *const u8, x.backward_actor as *const _ as *const u8);
            runtime.spawn_source(x)
        }
        tokio::signal::ctrl_c().await.unwrap();
    });

}

