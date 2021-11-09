use std::{collections::BTreeMap, sync::atomic::AtomicU64};

use serde::Deserialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

struct Component;

pub fn init() -> &'static dyn api::Component {
    &Component
}

impl api::Component for Component {
    fn create(&self, args: Vec<api::Argument>) -> api::Result<Box<api::Actor>> {
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

        Ok(Box::new(move |runtime, meta| {
            if runtime.is_source() && config.direction == "forward" {
                return Ok(Box::pin(listen(runtime, meta, config.port.unwrap())))
            }

            if config.direction == "forward" && meta.contains_key("tcp_stream_ptr") {
                let stream = unsafe { Box::from_raw(*meta["tcp_stream_ptr"].as_int().unwrap() as *mut TcpStream) };
                let (read_half, write_half) = stream.into_split();
                let mut meta = meta;
                meta.insert("tcp_stream_ptr".into(), (Box::into_raw(Box::new(write_half)) as u64).into());
                let next = runtime.spawn(0, meta);
                return Ok(Box::pin(forward(read_half, next)))
            }

            if config.direction == "backward" && meta.contains_key("tcp_stream_ptr") {
                let stream = unsafe { Box::from_raw(*meta["tcp_stream_ptr"].as_int().unwrap() as *mut tokio::net::tcp::OwnedWriteHalf) };
                return Ok(Box::pin(backward(runtime, stream)))
            }

            Err(anyhow::anyhow!("bug in tcp").into())
        }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
    }
}

async fn listen(runtime: Box<dyn api::Runtime>, meta: BTreeMap<String, api::ArgumentValue>, port: u16) -> api::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await.unwrap();
    let count = AtomicU64::new(0);

    loop { // TODO: graceful shutdown: somehow (e.g. through `runtime`) provide a globally shared condvar and use select! to cancel the event when that condvar changes
        match listener.accept().await {
            Ok((stream, addr)) => {
                eprintln!("Accepted connection from {:?}", addr);
                let mut meta = meta.clone();
                meta.insert("tcp_stream_ptr".into(), (Box::into_raw(Box::new(stream)) as u64).into()); // if the backward is never spawn, this is really a leak!
                meta.insert("tcp_origin_addr".into(), addr.to_string().into());
                meta.insert("tcp_stream_id".into(), count.fetch_add(1, std::sync::atomic::Ordering::SeqCst).into()); // TODO: assign another id to each actor, so the tuple (actor_id, stream_id) can be uniquely identify a stream
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
