use std::sync::atomic::AtomicU64;
use api::serde::Deserialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

struct Component;

struct Actor {
    has_output: bool
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config {
            port: Option<u16>,

            #[serde(default)]
            active: bool,

            outputs: Vec<String>,
            function_name: String,
        }

        let config: Config = api::parse_args(&arguments).unwrap();

        Box::new(Actor {
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => panic!("tcp can only accept one output")
            }
        })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: Box<R>, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        if self.has_output {
            todo!()
        }


        // match (runtime.is_source(), &config.direction[..], config.outputs.len()) {
        //     (true, "forward", _) => Ok(Box::pin(listen(runtime, meta, config.port.unwrap()))),
        //     (true, "backward", _) => Ok(Box::pin(async { Ok(()) })), // TODO: should this be an error?

        //     (false, "forward", len) => {
        //         let stream: Box<TcpStream> = meta.take("tcp_stream_ptr").unwrap();
        //         let (read_half, write_half) = stream.into_split();
        //         let mut meta = meta;
        //         meta.set("tcp_stream_ptr".into(), write_half);

        //         let next = match len {
        //             0 => runtime.spawn_conjugate(meta),
        //             1 => runtime.spawn(0, meta),
        //             _ => return Err(DispatchError::TooManyOutputs.into())
        //         };

        //         Ok(Box::pin(forward(read_half, next)))
        //     }

        //     (false, "backward", _) => {
        //         let stream: Box<tokio::net::tcp::OwnedWriteHalf> = meta.take("tcp_stream_ptr").unwrap();
        //         Ok(Box::pin(backward(runtime, stream)))
        //     }
        //     _ => Err(DispatchError::UnsupportedPosition.into())
        // }

        // runtime.spawn_task(self.write_stdout(mailbox.expect("no input")));
    }

    fn spawn_composite(&'static self, runtime: Box<R>, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_source(&'static self, runtime: Box<R>) {
        todo!()
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

// async fn forward(mut stream: (impl tokio::io::AsyncRead + Unpin), next: Box<dyn api::Address>) -> api::Result<()> {
//     let mut buffer = vec![0; 1024].into_boxed_slice();
//     loop {
//         let n = stream.read(&mut buffer[..]).await?;
//         if n == 0 { // EOF
//             return Ok(())
//         }

//         let fut = next.send(buffer[..n].iter().copied().collect());
//         fut.await;
//     }
// }

// async fn backward(mut runtime: Box<dyn api::Runtime>, mut stream: (impl tokio::io::AsyncWrite + Unpin)) -> api::Result<()> {
//     while let Some(msg) = runtime.read().await {
//         stream.write_all(&msg).await?
//     }
//     Ok(())
// }

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
