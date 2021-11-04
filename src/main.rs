#![feature(new_uninit)]
#![feature(box_into_pin)]
#![feature(never_type)]

use std::collections::BTreeSet;

use oh_my_rust::*;

use anyhow::{Context, Result};

mod script;
mod runtime;

/// A component with runtime-tracked states
struct Node {
    comp: &'static dyn api::Component,
    outputs: &'static [usize],
    conj: usize
}

#[allow(clippy::vec_init_then_push)]
fn main() -> Result<!> {
    let mut components = vec![];

    #[cfg(feature="stdio")]
    components.push(stdio::init());

    #[cfg(feature="xor")]
    components.push(xor::init());

    #[cfg(feature="tcp")]
    components.push(tcp::init());


    let nodes: &_ = script::load_script(r#"tcp(2222) => xor("fuck") => xor()"#, &components).unwrap().leak();

    let runtime = runtime::Runtime::new(nodes).box_and_leak();

    let tokio_rt = tokio::runtime::Runtime::new()?;

    tokio_rt.block_on(async move {
        let non_source: BTreeSet<_> = nodes.iter()
            .flat_map(|x| x.outputs.iter())
            .copied().collect();
        let tasks: Vec<_> = nodes.iter().enumerate()
            .filter(|(i, _)| !non_source.contains(i))
            .map(|(_, x)| runtime.spawn(x))
            .map(|actor| tokio::spawn(actor.run()))
            .collect();
        for task in tasks {
            task.await.unwrap()
        }
    });

    unreachable!();
}

