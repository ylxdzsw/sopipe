use std::sync::atomic::AtomicU64;
use api::serde::Deserialize;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}};

struct Component;

mod listen;
mod active;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(crate="api::serde")]
struct Config<'a> {
    port: Option<u16>,

    addr: Option<&'a str>,

    #[serde(default)]
    active: bool,

    outputs: Vec<&'a str>,
    function_name: String,
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let config: Config = api::parse_args(&arguments).unwrap();

        if config.active {
            todo!()
        } else {
            Box::new(listen::Actor::new(config))
        }
    }

    fn functions(&self) -> &'static [&'static str] {
        &["tcp"]
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
