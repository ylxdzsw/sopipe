use super::*;

pub struct Actor {
    pub has_output: bool
}

impl Actor {
    pub(crate) fn new(config: Config) -> Self {
        listen::Actor {
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => panic!("tcp can only accept one output")
            }
        }
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, _runtime: Box<R>, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_composite(&'static self, _runtime: Box<R>, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_source(&'static self, runtime: Box<R>) {
        if self.has_output {

        } else { // just echo
            todo!()
        }

        // match (runtime.is_source(), &config.direction[..], config.outputs.len()) {
        //     (true, "forward", _) => Ok(Box::pin(listen(runtime, meta, config.port.unwrap()))),
        //     (true, "backward", _) => Ok(Box::pin(async { Ok(()) })), // TODO: should this be an error?

        //     (false, "forward", len) => {

        //     }

        //     (false, "backward", _) => {
        //         let stream: Box<tokio::net::tcp::OwnedWriteHalf> = meta.take("tcp_stream_ptr").unwrap();
        //         Ok(Box::pin(backward(runtime, stream)))
        //     }
        //     _ => Err(DispatchError::UnsupportedPosition.into())
        // }

        // runtime.spawn_task(self.write_stdout(mailbox.expect("no input")));
    }
}

async fn listen(runtime: impl api::Runtime, port: u16) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    let count = AtomicU64::new(0);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                eprintln!("Accepted connection from {:?}", addr);
                let mut meta = api::MetaData::default();
                meta.set("tcp_origin_addr".into(), addr);
                meta.set("tcp_stream_id".into(), count.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
                // runtime.spawn_next(0, meta, );
            },
            Err(err) => {
                eprintln!("accept error = {}", err)
            }
        }
    }
}

async fn forward(mut stream: impl AsyncReadExt + Unpin, mut addr: impl api::Address) {
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
