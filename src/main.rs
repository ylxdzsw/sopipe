use std::{
    collections::BTreeSet,
    sync::atomic::{AtomicU32, Ordering},
};

mod runtime;
mod script;

type R = runtime::RuntimeHandler;

struct Counter(&'static AtomicU32);
impl Counter {
    fn new(a: &'static AtomicU32) -> Self {
        a.fetch_add(1, Ordering::Relaxed);
        Counter(a)
    }
}
impl Drop for Counter {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

/// An (composite) actor with runtime-tracked states
struct Node {
    forward_actor: &'static dyn api::Actor<R>,
    backward_actor: &'static dyn api::Actor<R>,
    outputs: &'static [usize],

    task_count: AtomicU32,
}

impl Node {
    fn new(
        forward_actor: &'static dyn api::Actor<R>,
        backward_actor: &'static dyn api::Actor<R>,
        outputs: &'static [usize],
    ) -> Self {
        Self {
            forward_actor,
            backward_actor,
            outputs,
            task_count: Default::default(),
        }
    }
}

fn main() {
    let components = vec![
        #[cfg(feature = "aead")]
        aead::init(),

        #[cfg(feature = "auth")]
        auth::init(),

        #[cfg(feature = "balance")]
        balance::init(),

        #[cfg(feature = "drop")]
        drop::init(),

        #[cfg(feature = "echo")]
        echo::init(),

        #[cfg(feature = "exec")]
        exec::init(),

        #[cfg(feature = "http2")]
        http2::init(),

        #[cfg(feature = "miniz")]
        miniz::init(),

        #[cfg(feature = "socks5")]
        socks5::init(),

        #[cfg(feature = "stdio")]
        stdio::init(),

        #[cfg(feature = "tcp")]
        tcp::init(),

        #[cfg(feature = "tee")]
        tee::init(),

        #[cfg(feature = "throttle")]
        throttle::init(),

        #[cfg(feature = "udp")]
        udp::init(),

        #[cfg(feature = "vmess")]
        vmess::init(),

        #[cfg(feature = "xor")]
        xor::init(),
    ];

    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        print!("Sopipe {}", option_env!("CARGO_PKG_VERSION").unwrap_or_default());
        for comp in components.iter() {
            print!(" {}", comp.name())
        }
        std::process::exit(0);
    }

    let nodes: &_ = script::Interpreter::load_script(&args[1], &components).leak();

    let runtime = Box::leak(Box::new(runtime::Runtime::new(nodes)));

    let tokio_rt = tokio::runtime::Runtime::new().unwrap();

    tokio_rt.block_on(async move {
        runtime.set_run_level(api::RunLevel::Init);

        let not_source: BTreeSet<_> = nodes.iter().flat_map(|x| x.outputs.iter()).copied().collect();
        for (i, x) in nodes.iter().enumerate() {
            if not_source.contains(&i) {
                continue;
            }
            assert_eq!(
                x.forward_actor as *const _ as *const u8,
                x.backward_actor as *const _ as *const u8
            );
            runtime.spawn_source(x)
        }

        // TODO: actually wait for all tasks to finish Init
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        runtime.set_run_level(api::RunLevel::Run);

        tokio::spawn(async {
            tokio::signal::ctrl_c().await.unwrap();
            runtime.set_run_level(api::RunLevel::Shut);
            eprintln!(
                "SIGINT recieved. Stoping accepting new connections.\n\
                       Waiting for exiting tasks. Press Ctrl+C again to force exit."
            );

            tokio::signal::ctrl_c().await.unwrap();
            eprintln!("SIGINT recieved. Aborting.");
            std::process::exit(1);
        });

        // Silently exit when no task runinng. Long-running tasks like tcp listening won't die unless runlevel enters Shut.
        while nodes.iter().any(|node| node.task_count.load(Ordering::Relaxed) != 0) {
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await
        }
    });
}
