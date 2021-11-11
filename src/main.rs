use std::collections::BTreeSet;

mod script;
mod runtime;

/// An (composite) actor with runtime-tracked states
struct Node {
    forward_actor: &'static dyn api::Actor,
    backward_actor: &'static dyn api::Actor,
    outputs: &'static [usize],
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

    let tokio_rt = api::tokio::runtime::Runtime::new().unwrap();

    tokio_rt.block_on(async move {
        let non_source: BTreeSet<_> = nodes.iter()
            .flat_map(|x| x.outputs.iter())
            .copied().collect();
        for (i, x) in nodes.iter().enumerate() {
            if non_source.contains(&i) {
                continue
            }
            assert_eq!(x.forward_actor as *const _ as *const u8, x.backward_actor as *const _ as *const u8);
            runtime.spawn(x)
        }
        api::tokio::signal::ctrl_c().await.unwrap();
    });

}

