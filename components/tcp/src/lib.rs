use std::sync::atomic::AtomicU64;
use thiserror::Error;
use serde::Deserialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

#[derive(Error, Debug)]
enum DispatchError {
    #[error("tcp only has one output. Use a router componenet.")]
    TooManyOutputs,
    #[error("tcp can only be used as source or sink nodes.")]
    UnsupportedPosition,
}

struct Component;

pub fn init() -> &'static dyn api::Component {
    &Component
}

impl api::Component for Component {
    fn create(&self, args: Vec<(String, api::Argument)>) -> api::Result<Box<api::Actor>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Config {
            port: Option<u16>,

            #[serde(default)]
            active: bool,

            direction: String,
            outputs: Vec<String>,
            function_name: String,
        }

        let config: Config = api::helper::parse_args(&args).unwrap();

        Ok(Box::new(move |runtime, mut meta| {
            match (runtime.is_source(), &config.direction[..], config.outputs.len()) {
                (true, "forward", _) => Ok(Box::pin(listen(runtime, meta, config.port.unwrap()))),
                (true, "backward", _) => Ok(Box::pin(async { Ok(()) })), // TODO: should this be an error?

                (false, "forward", len) => {
                    let stream: Box<TcpStream> = meta.take("tcp_stream_ptr").unwrap();
                    let (read_half, write_half) = stream.into_split();
                    let mut meta = meta;
                    meta.set("tcp_stream_ptr".into(), write_half);

                    let next = match len {
                        0 => runtime.spawn_conjugate(meta),
                        1 => runtime.spawn(0, meta),
                        _ => return Err(DispatchError::TooManyOutputs.into())
                    };

                    Ok(Box::pin(forward(read_half, next)))
                }

                (false, "backward", _) => {
                    let stream: Box<tokio::net::tcp::OwnedWriteHalf> = meta.take("tcp_stream_ptr").unwrap();
                    Ok(Box::pin(backward(runtime, stream)))
                }
                _ => Err(DispatchError::UnsupportedPosition.into())
            }
        }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

async fn listen(runtime: Box<dyn api::Runtime>, meta: api::MetaData, port: u16) -> api::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    let count = AtomicU64::new(0);

    loop { // TODO: graceful shutdown: somehow (e.g. through `runtime`) provide a globally shared condvar and use select! to cancel the event when that condvar changes
        match listener.accept().await {
            Ok((stream, addr)) => {
                eprintln!("Accepted connection from {:?}", addr);
                let mut meta = meta.clone();
                meta.set("tcp_stream_ptr".into(), stream);
                meta.set("tcp_origin_addr".into(), addr);
                meta.set("tcp_stream_id".into(), count.fetch_add(1, std::sync::atomic::Ordering::SeqCst)); // TODO: assign another id to each actor, so the tuple (actor_id, stream_id) can be uniquely identify a stream
                runtime.spawn_self(meta);
            },
            Err(err) => {
                eprintln!("accept error = {:?}", err)
            }
        }
    }
}

async fn forward(mut stream: (impl tokio::io::AsyncRead + Unpin), next: Box<dyn api::Address>) -> api::Result<()> {
    let mut buffer = vec![0; 1024].into_boxed_slice();
    loop {
        let n = stream.read(&mut buffer[..]).await?;
        if n == 0 { // EOF
            return Ok(())
        }

        let fut = next.send(buffer[..n].iter().copied().collect());
        fut.await;
    }
}

async fn backward(mut runtime: Box<dyn api::Runtime>, mut stream: (impl tokio::io::AsyncWrite + Unpin)) -> api::Result<()> {
    while let Some(msg) = runtime.read().await {
        stream.write_all(&msg).await?
    }
    Ok(())
}
