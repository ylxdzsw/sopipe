pub use serde; // expose to components. Note additional attr must be used https://github.com/serde-rs/serde/issues/1465#issuecomment-800686252

mod argument;
pub use argument::Argument;

mod parser;
pub use parser::parse_args;

mod metadata;
pub use metadata::MetaData;

mod runtime;
pub use runtime::{Runtime, Address, Mailbox};


pub trait Actor<R: Runtime>: Sync {
    /// spawn an instance of this actor, handling messages in the mailbox and send responses to the address.
    fn spawn(&'static self, runtime: Box<R>, metadata: MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>);

    /// spawn an instance of this actor as a part in a composited component. It acts like a one-way pipe that process messages from the mailbox and send to the address.
    fn spawn_composite(&'static self, runtime: Box<R>, metadata: MetaData, address: Option<R::Address>, mailbox: Option<R::Mailbox>);

    /// spawn an instance of this actor as a source node
    fn spawn_source(&'static self, runtime: Box<R>);
}

/// The main trait for components.
pub trait Component<R: Runtime>: Sync {
    /// get the name of functions this component registers
    fn functions(&'static self) -> &'static [&'static str];

    /// create an instance for a node in the pipeline.
    /// the arguments includes user-provided arguments as well as the following:
    /// function_name (String): the name of function in the user script
    /// outputs (List<String>): the names of outputs. Unamed outputs have empty names.
    fn create(&'static self, arguments: Vec<(String, Argument)>) -> Box<dyn Actor<R>>;
}


