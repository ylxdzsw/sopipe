use std::time::Duration;

use tokio::net::ToSocketAddrs;

use super::*;

pub struct Actor {
    addr: Option<String>,
    port: Option<u16>,
    has_output: bool
}

impl Actor {
    pub(crate) fn new(config: Config) -> Self {
        let (addr, port) = config.get_addr_and_port();

        Actor {
            addr, port,
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => panic!("tcp can only accept one output")
            }
        }
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, mut metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        assert!(!self.has_output);

        let mut addr = metadata.take::<String>("destination_addr").map(|x| *x);
        let mut port = metadata.take::<u16>("destination_port").map(|x| *x);

        if addr.is_some() || port.is_some() {
            if self.addr.is_some() || self.port.is_some() {
                panic!("The stream already contains destination information")
            }
        } else {
            addr = self.addr.clone();
            port = self.port;
        }

        if let Some(port) = port {
            runtime.spawn_task_with_runtime(move |runtime| self.connect(runtime, (addr.unwrap(), port) , address.unwrap(), mailbox.unwrap()))
        } else {
            runtime.spawn_task_with_runtime(move |runtime| self.connect(runtime, addr.unwrap(), address.unwrap(), mailbox.unwrap()))
        }
    }

    fn spawn_source(&'static self, runtime: R) {
        assert!(self.has_output);
        runtime.spawn_task_with_runtime(move |runtime| self.listen(runtime))
    }
}

impl Actor {
    async fn connect(&self, runtime: impl api::Runtime, dest: impl ToSocketAddrs, address: impl api::Address, mailbox: impl api::Mailbox) {
        match tokio::net::TcpStream::connect(dest).await {
            Ok(stream) => {
                let (reader, writer) = stream.into_split();
                runtime.spawn_task(read_tcp(reader, address));
                runtime.spawn_task(write_tcp(writer, mailbox));
            },
            Err(e) => {
                eprintln!("connection error = {}", e);
                // what to do? retry?
            },
        }
    }

    async fn listen(&self, runtime: impl api::Runtime) {
        let addr = self.addr.as_deref().unwrap_or("0.0.0.0");
        let listener = if let Some(port) = self.port {
            TcpListener::bind((addr, port)).await.unwrap()
        } else {
            TcpListener::bind(addr).await.unwrap()
        };

        let count = AtomicU64::new(0);

        while let api::RunLevel::Init = runtime.get_runlevel() {
            tokio::time::sleep(Duration::from_millis(20)).await
        }

        while let api::RunLevel::Run = runtime.get_runlevel() {
            match tokio::time::timeout(Duration::from_secs(1), listener.accept()).await {
                Ok(Ok((stream, origin))) => {
                    eprintln!("Accepted connection from {:?}", origin);
                    let mut meta = api::MetaData::default();
                    meta.set("stream_type".into(), "TCP".to_string());
                    meta.set("origin_addr".into(), origin);
                    meta.set("stream_id".into(), count.fetch_add(1, std::sync::atomic::Ordering::Relaxed));

                    let (reader, writer) = stream.into_split();
                    let (forward_address, forward_mailbox) = runtime.channel();
                    let (backward_address, backward_mailbox) = runtime.channel();
                    runtime.spawn_next(0, meta, backward_address, forward_mailbox);
                    runtime.spawn_task(read_tcp(reader, forward_address));
                    runtime.spawn_task(write_tcp(writer, backward_mailbox));
                },
                Ok(Err(err)) => {
                    eprintln!("accept error = {}", err)
                }
                Err(_) => {} // timeout, check runlevel and listen again
            }
        }
    }
}
