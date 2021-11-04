use std::{collections::BTreeMap, error::Error, ptr::NonNull};
use thiserror::Error;

struct Spec {

}


#[derive(Copy, Clone)]
enum Component { STDIN, STDOUT, STDIO }

impl api::Component for Component {
    fn spawn(&self, runtime: Box<dyn api::Runtime>, args: BTreeMap<String, api::ArgumentValue>) -> Result<Box<dyn api::Actor>, Box<dyn Error + Send + Sync>> {
        todo!()
    }

}


#[derive(Error, Debug)]
pub enum TcpError {
    #[error("Invalid arguments. Detail: {0}")]
    InvalidArgument(&'static str),
}

struct Actor {

}

#[api::async_trait]
impl api::Actor for Actor {
    async fn run(self: Box<Self>, ) {}
}

impl api::ComponentSpec for Spec {
    fn create(&self, arguments: Vec<api::Argument>) -> Result<Box<dyn api::Component>, Box<dyn Error + Send + Sync>> {
        let comp = match &arguments.iter().find(|x| x.0 == "function_name").unwrap().1.as_string().unwrap()[..] {
            "stdin" => Component::STDIN,
            "stdout" => Component::STDOUT,
            "stdio" => Component::STDIO,
            _ => unreachable!()
        };

        Ok(Box::new(comp))
    }

    fn functions(&self) -> &'static [&'static str] {
        &["stdin", "stdout", "stdio"]
    }

}

pub fn init() -> &'static dyn api::ComponentSpec {
    println!("Hello, world from tcp");
    &Spec {}
}
