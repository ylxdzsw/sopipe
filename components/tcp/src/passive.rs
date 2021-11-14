use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use super::*;

pub struct Actor {
    addr: Option<IpAddr>,
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

        let dest = if let Some(dest) = metadata.take::<SocketAddr>("tcp_destination") {
            if self.addr.is_some() || self.port.is_some() {
                panic!("The stream already contains destination information")
            }
            *dest
        } else {
            SocketAddr::new(self.addr.unwrap(), self.port.unwrap())
        };

        runtime.spawn_task_with_runtime(move |runtime| self.connect(runtime, dest, address.unwrap(), mailbox.unwrap()))
    }

    fn spawn_source(&'static self, runtime: R) {
        runtime.spawn_task_with_runtime(move |runtime| self.listen(runtime))
    }
}

impl Actor {
    async fn connect(&self, runtime: impl api::Runtime, dest: SocketAddr, address: impl api::Address, mailbox: impl api::Mailbox) {
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
        let addr = self.addr.unwrap_or_else(|| "0.0.0.0".parse().unwrap());
        let listener = TcpListener::bind(SocketAddr::new(addr, self.port.unwrap())).await.unwrap();
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
                    meta.set("tcp_origin_addr".into(), origin);
                    meta.set("tcp_stream_id".into(), count.fetch_add(1, std::sync::atomic::Ordering::Relaxed));

                    let (reader, writer) = stream.into_split();
                    if self.has_output {
                        let (forward_address, forward_mailbox) = runtime.channel();
                        let (backward_address, backward_mailbox) = runtime.channel();
                        runtime.spawn_next(0, meta, backward_address, forward_mailbox);
                        runtime.spawn_task(read_tcp(reader, forward_address));
                        runtime.spawn_task(write_tcp(writer, backward_mailbox));
                    } else { // just echo
                        todo!()
                    }
                },
                Ok(Err(err)) => {
                    eprintln!("accept error = {}", err)
                }
                Err(_) => {} // timeout, check runlevel and listen again
            }
        }
    }
}
