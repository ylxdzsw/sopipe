use api::serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct Component;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
enum FuncName { STDIN, STDOUT, STDIO }

struct Actor {
    func: FuncName,
    has_output: bool,
    no_flush: bool,
    buffer_size: usize,
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&self, args: Vec<(String, api::Argument)>) -> api::Result<Box<dyn api::Actor<R>>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            direction: &'a str,
            outputs: Vec<String>,
            function_name: &'a str,
            #[serde(default)]
            no_flush: bool,
        }

        let config: Config = api::parse_args(&args).unwrap();

        Ok(Box::new(Actor {
            func: match config.function_name {
                "stdin" => FuncName::STDIN,
                "stdout" => FuncName::STDOUT,
                "stdio" => FuncName::STDIO,
                _ => unreachable!()
            },
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => return Err(api::Error::misuse("too many outputs", None))
            },
            no_flush: config.no_flush,
            buffer_size: 1024
        }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["stdin", "stdout", "stdio"]
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: Box<R>, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        let (forward_address, forward_mailbox) = runtime.channel();
    }

    fn spawn_composite(&'static self, runtime: Box<R>, metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_source(&'static self, runtime: Box<R>) {
        todo!()
    }
}

impl Actor {
    async fn read_stdin(&self, mut addr: impl api::Address) -> api::Result<()> {
        let mut stdin = tokio::io::stdin();
        let mut buffer = vec![0; self.buffer_size].into_boxed_slice();

        loop {
            let n = stdin.read(&mut buffer[..]).await.map_err(|e| api::Error::non_fatal("failed reading stdin", Some(Box::new(e))))?;
            if n == 0 { // EOF
                return Ok(())
            }

            addr.send(buffer[..n].iter().copied().collect()).await;
        }
    }

    async fn write_stdout(&self, mut mail: impl api::Mailbox) -> api::Result<()> {
        let mut stdout = tokio::io::stdout();
        while let Some(packet) = mail.recv().await {
            stdout.write_all(&packet).await.map_err(|e| api::Error::non_fatal("failed writing stdout", Some(Box::new(e))))?;
            if !self.no_flush {
                stdout.flush().await.map_err(|e| api::Error::non_fatal("failed flushing stdout", Some(Box::new(e))))?
            }
        }
        Ok(())
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
