use api::serde::Deserialize;
use tokio::io::{Result, AsyncReadExt, AsyncWriteExt};

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
    fn create(&self, args: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        #[serde(crate="api::serde")]
        struct Config<'a> {
            outputs: Vec<String>,
            function_name: &'a str,
            #[serde(default)]
            no_flush: bool,
        }

        let config: Config = api::parse_args(&args).unwrap();

        Box::new(Actor {
            func: match config.function_name {
                "stdin" => FuncName::STDIN,
                "stdout" => FuncName::STDOUT,
                "stdio" => FuncName::STDIO,
                _ => unreachable!()
            },
            has_output: match config.outputs.len() {
                0 => false,
                1 => true,
                _ => panic!("too many outputs")
            },
            no_flush: config.no_flush,
            buffer_size: 1024
        })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["stdin", "stdout", "stdio"]
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        if self.has_output {
            todo!()
        }

        if let FuncName::STDIN | FuncName::STDIO = self.func {
            panic!("sink node can only be stdout")
        }

        runtime.spawn_task(self.write_stdout(mailbox.expect("no input")));
    }

    fn spawn_composite(&'static self, _runtime: R, _metadata: api::MetaData, _address: Option<R::Address>, _mailbox: Option<R::Mailbox>) {
        todo!()
    }

    fn spawn_source(&'static self, runtime: R) {
        if let FuncName::STDOUT = self.func {
            panic!("misuse")
        }

        if self.has_output {
            let (forward_address, forward_mailbox) = runtime.channel();
            let (backward_address, backward_mailbox) = runtime.channel();
            runtime.spawn_next(0, Default::default(), backward_address, forward_mailbox);
            runtime.spawn_task(self.read_stdin(forward_address));
            runtime.spawn_task(self.write_stdout(backward_mailbox));
        } else { // trivial case: direct echo
            let (address, mailbox) = runtime.channel();
            runtime.spawn_task(self.read_stdin(address));
            runtime.spawn_task(self.write_stdout(mailbox));
        }
    }
}

impl Actor {
    async fn read_stdin(&self, mut addr: impl api::Address) -> Result<()> {
        let mut stdin = tokio::io::stdin();
        let mut buffer = vec![0; self.buffer_size].into_boxed_slice();

        loop {
            let n = stdin.read(&mut buffer[..]).await?;
            if n == 0 { // EOF
                return Ok(())
            }

            #[allow(clippy::question_mark)]
            if addr.send(buffer[..n].iter().copied().collect()).await.is_err() {
                return Ok(())
            }
        }
    }

    async fn write_stdout(&self, mut mail: impl api::Mailbox) -> Result<()> {
        let mut stdout = tokio::io::stdout();
        while let Some(packet) = mail.recv().await {
            stdout.write_all(&packet).await?;
            if !self.no_flush {
                stdout.flush().await?
            }
        }
        Ok(())
    }
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
