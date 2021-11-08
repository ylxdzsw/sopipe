use std::{collections::BTreeMap};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::Deserialize;

struct Spec;

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
enum FuncName { STDIN, STDOUT, STDIO }

struct Config {
    func: FuncName,
    no_flush: bool
}

impl api::ComponentSpec for Spec {
    fn create(&self, args: Vec<api::Argument>) -> api::Result<api::ActorFactory> {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize)]
        struct _Config<'a> {
            direction: &'a str,
            outputs: Vec<String>,
            function_name: &'a str,
            #[serde(default)]
            no_flush: bool,
        }

        let _config: _Config = api::helper::parse_args(&args).unwrap();

        let func = match _config.function_name {
            "stdin" => FuncName::STDIN,
            "stdout" => FuncName::STDOUT,
            "stdio" => FuncName::STDIO,
            _ => unreachable!()
        };

        let config = &*Box::leak(Box::new(Config { func, no_flush: _config.no_flush }));

        Ok(Box::new(move |runtime, meta| {
            Ok(Box::new(move || Box::pin(run(config, runtime, meta))))
        }))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["stdin", "stdout", "stdio"]
    }
}

async fn run(config: &Config, mut runtime: Box<dyn api::Runtime>, meta: BTreeMap<String, api::ArgumentValue>) -> api::Result<()> {
    if runtime.is_source() && matches!(config.func, FuncName::STDIN | FuncName::STDIO) {
        let mut stdin = tokio::io::stdin();
        let mut buffer = vec![0; 1024].into_boxed_slice();

        let next = runtime.spawn(0, meta);

        loop {
            let n = stdin.read(&mut buffer[..]).await?;
            if n == 0 { // EOF
                return Ok(())
            }

            let fut = next.send(buffer[..n].iter().copied().collect()); // to drop &next before awaiting
            fut.await;
        }
    } else if !runtime.is_source() && matches!(config.func, FuncName::STDOUT | FuncName::STDIO) {
        let mut stdout = tokio::io::stdout();
        while let Some(packet) = runtime.read().await {
            stdout.write(&packet).await?;
            if !config.no_flush {
                stdout.flush().await?
            }
        }
    }

    Ok(())
}

pub fn init() -> &'static dyn api::ComponentSpec {
    &Spec {}
}
