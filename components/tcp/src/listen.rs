use tokio::time::Duration;

use super::*;

pub struct Actor {
    port: u16,
    has_output: bool
}

impl Actor {
    pub(crate) fn new(config: Config) -> Self {
        listen::Actor {
            port: config.port.unwrap(),
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => panic!("tcp can only accept one output")
            }
        }
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, _runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_composite(&'static self, _runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_source(&'static self, runtime: R) {
        runtime.spawn_task_with_runtime(move |runtime| self.listen(runtime))
    }
}

impl Actor {
    async fn listen(&self, runtime: impl api::Runtime) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port)).await.unwrap();
        let count = AtomicU64::new(0);

        while let api::RunLevel::Init = runtime.get_runlevel() {
            tokio::time::sleep(Duration::from_millis(20)).await
        }

        while let api::RunLevel::Run = runtime.get_runlevel() {
            match tokio::time::timeout(Duration::from_secs(1), listener.accept()).await {
                Ok(Ok((stream, origin))) => {
                    eprintln!("Accepted connection from {:?}", origin);
                    let mut meta = api::MetaData::default();
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

async fn read_tcp(mut stream: impl AsyncReadExt + Unpin, mut addr: impl api::Address) {
    let mut buffer = vec![0; 1024].into_boxed_slice();
    loop {
        match stream.read(&mut buffer[..]).await {
            Ok(0) => return, // EOF
            Ok(n) => if addr.send(buffer[..n].iter().copied().collect()).await.is_err() {
                return
            }
            Err(e) => {
                eprintln!("IO error: {}", e);
                return
            }
        }
    }
}

async fn write_tcp(mut stream: impl AsyncWriteExt + Unpin, mut mail: impl api::Mailbox) {
    while let Some(msg) = mail.recv().await {
        if stream.write_all(&msg).await.is_err() {
            break
        }
    }
}
