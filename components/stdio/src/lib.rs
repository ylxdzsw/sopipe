use std::{collections::BTreeMap, error::Error};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::Deserialize;

struct Spec;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
enum FuncName { STDIN, STDOUT, STDIO }

struct Component {
    func: FuncName,
    no_flush: bool,
}

struct Actor {
    runtime: Box<dyn api::Runtime>,
    comp: &'static Component,
    meta: BTreeMap<String, api::ArgumentValue>
}

impl api::ComponentSpec for Spec {
    fn create(&self, args: Vec<api::Argument>) -> Result<Box<dyn api::Component>, Box<dyn Error + Send + Sync>> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct Config<'a> {
            direction: &'a str,
            outputs: Vec<String>,
            function_name: &'a str,
            #[serde(default)]
            no_flush: bool,
        }

        let config: Config = api::helper::parse_args(&args).unwrap();

        let func = match config.function_name {
            "stdin" => FuncName::STDIN,
            "stdout" => FuncName::STDOUT,
            "stdio" => FuncName::STDIO,
            _ => unreachable!()
        };
        Ok(Box::new(Component { func, no_flush: config.no_flush }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["stdin", "stdout", "stdio"]
    }
}

impl api::Component for Component {
    fn spawn(&'static self, runtime: Box<dyn api::Runtime>, meta: BTreeMap<String, api::ArgumentValue>) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        Ok(Box::new(Actor { runtime, comp: self, meta}))
    }
}

async fn run(mut actor: Box<Actor>) -> anyhow::Result<()> {
    if actor.runtime.is_source() && matches!(actor.comp.func, FuncName::STDIN | FuncName::STDIO) {
        let mut stdin = tokio::io::stdin();
        let mut buffer = vec![0; 1024].into_boxed_slice();

        let next = actor.runtime.spawn(0, actor.meta);

        loop {
            let n = stdin.read(&mut buffer[..]).await?;
            if n == 0 { // EOF
                return Ok(())
            }

            let fut = next.send(buffer[..n].iter().copied().collect()); // to drop &next before awaiting
            fut.await;
        }
    } else if !actor.runtime.is_source() && matches!(actor.comp.func, FuncName::STDOUT | FuncName::STDIO) {
        let mut stdout = tokio::io::stdout();
        while let Some(packet) = actor.runtime.read().await {
            stdout.write(&packet).await?;
            if !actor.comp.no_flush {
                stdout.flush().await?
            }
        }
    }

    Ok(())
}

#[api::async_trait]
impl api::Actor for Actor {
    async fn run(self: Box<Self>) -> Result<(), Box<dyn Error + Send + Sync>> {
        run(self).await.map_err(|e| e.into())
    }
}


pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
