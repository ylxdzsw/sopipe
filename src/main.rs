use std::collections::BTreeSet;

mod script;
mod runtime;

/// An actor with runtime-tracked states
struct Node {
    actor: &'static api::Actor,
    outputs: &'static [usize],
    conj: usize
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

    let nodes: &_ = script::load_script(&args[1], &components).unwrap().leak();

    let runtime = Box::leak(Box::new(runtime::Runtime::new(nodes)));

    let tokio_rt = tokio::runtime::Runtime::new().unwrap();

    tokio_rt.block_on(async move {
        let non_source: BTreeSet<_> = nodes.iter()
            .flat_map(|x| x.outputs.iter())
            .copied().collect();
        let tasks: Vec<_> = nodes.iter().enumerate()
            .filter(|(i, _)| !non_source.contains(i))
            .map(|(_, x)| runtime.spawn(x))
            .collect();
        for task in tasks {
            task.await.unwrap().unwrap()
        }
    });

    unreachable!(); // TODO: add a global graceful exit flag in Runtime
}

