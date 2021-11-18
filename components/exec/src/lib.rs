use std::sync::Arc;

use tokio::io::{Result, AsyncReadExt, AsyncWriteExt};

struct Component;

struct Actor {
    has_output: bool,
    args: Vec<String>
}

impl<R: api::Runtime> api::Component<R> for Component {
    fn create(&'static self, arguments: Vec<(String, api::Argument)>) -> Box<dyn api::Actor<R>> {
        let mut n_outputs = usize::MAX;
        let mut args = vec![];

        for (name, value) in arguments {
            match &name[..] {
                "" => if let api::Argument::String(x) = value {
                    args.push(x)
                } else {
                    panic!("wrong argument type")
                },
                "outputs" => n_outputs = value.as_vec().unwrap().len(),
                "function_name" => {}
                _ => panic!("wrong argument")
            }
        }

        Box::new(Actor {
            args,
            has_output: match n_outputs {
                0 => false,
                1 => true,
                _ => panic!("too many outputs")
            }
        })
    }

    fn functions(&self) -> &'static [&'static str] {
        &["exec"]
    }
}

impl<R: api::Runtime> api::Actor<R> for Actor {
    fn spawn(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        assert!(!self.has_output);

        match self.spawn() {
            Ok(child) => {
                pipe_exec(runtime, child, address.unwrap(), mailbox.unwrap());
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    }

    fn spawn_composite(&'static self, runtime: R, _metadata: api::MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>) {
        match self.spawn() {
            Ok(child) => {
                pipe_exec(runtime, child, address.unwrap(), mailbox.unwrap());
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    }

    fn spawn_source(&'static self, runtime: R) {
        assert!(self.has_output);

        let (forward_address, forward_mailbox) = runtime.channel();
        let (backward_address, backward_mailbox) = runtime.channel();
        runtime.spawn_next(0, Default::default(), backward_address, forward_mailbox);

        match self.spawn() {
            Ok(child) => {
                pipe_exec(runtime, child, forward_address, backward_mailbox);
            },
            Err(e) => {
                eprintln!("{}", e);
            },
        }
    }
}

impl Actor {
    fn spawn(&self) -> Result<tokio::process::Child> {
        tokio::process::Command::new(&self.args[0])
            .args(&self.args[1..])
            .kill_on_drop(true)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
    }
}

fn pipe_exec(runtime: impl api::Runtime, mut child: tokio::process::Child, mut addr: impl api::Address, mut mail: impl api::Mailbox) -> Option<()> {
    let mut child_stdin = child.stdin.take()?;
    let mut child_stdout = child.stdout.take()?;

    let child_rc_1 = Arc::new(child); // we use kill_on_drop to clean up
    let child_rc_2 = child_rc_1.clone();

    runtime.spawn_task(async move {
        let _alive = child_rc_1;
        while let Some(msg) = mail.recv().await {
            if let Err(e) = child_stdin.write_all(&msg).await {
                eprintln!("error writing child process: {}", e);
                return
            }
        }
    });

    runtime.spawn_task(async move {
        let _alive = child_rc_2;
        let mut buf = vec![0; 65536];
        loop {
            match child_stdout.read(&mut buf).await {
                Ok(0) => return, // EOF
                Ok(n) => {
                    if addr.send(Box::from(&buf[..n])).await.is_err() {
                        return
                    }
                },
                Err(e) => {
                    eprintln!("error reading child process: {}", e);
                    return
                },
            }
        }
    });

    None
}

pub fn init<R: api::Runtime>() -> &'static dyn api::Component<R> {
    &Component {}
}
