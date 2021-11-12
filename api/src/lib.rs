use std::{any::Any, collections::BTreeMap, error::Error, future::Future, pin::Pin, sync::Arc};
pub use serde; // expose to components. Note additional attr need to be used https://github.com/serde-rs/serde/issues/1465#issuecomment-800686252
pub mod helper; // helper lib for components

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

/// An enum type that represents user arguments
#[derive(Debug, Clone)]
pub enum Argument {
    String(String),
    Int(u64),
    Vec(Vec<Argument>),
    None
}

impl From<String> for Argument {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}

impl From<u64> for Argument {
    fn from(x: u64) -> Self {
        Self::Int(x)
    }
}

impl<T> FromIterator<T> for Argument where Argument: std::convert::From<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        Self::Vec(iter.into_iter().map(Argument::from).collect())
    }
}

impl Argument {
    pub fn type_name(&self) -> &'static str {
        match &self {
            Argument::String(_) => "string",
            Argument::Int(_) => "int",
            Argument::Vec(_) => "vec",
            Argument::None => "none",
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match &self {
            &Argument::String(x) => Some(x),
            _ => None
        }
    }

    pub fn as_int(&self) -> Option<&u64> {
        match &self {
            &Argument::Int(x) => Some(x),
            _ => None
        }
    }

    pub fn as_vec(&self) -> Option<&[Argument]> {
        match &self {
            &Argument::Vec(x) => Some(x),
            _ => None
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, &Argument::None)
    }
}

pub trait Address: dyn_clone::DynClone + Send + Sync {
    fn send(&mut self, msg: Box<[u8]>) -> Pin<Box<dyn Future<Output=()>>>;
}

pub trait Mailbox: Send + Sync {
    fn recv(&mut self) -> Pin<Box<dyn Future<Output=Option<Box<[u8]>>>>>;
}

/// A trait that provides runtime functions to components. It is tied to each actor.
pub trait Runtime: Sync + Send {
    type Address: Address;
    type Mailbox: Mailbox;

    /// spawn an actor of the i-th output
    /// metadata provides information about this stream
    /// address allows the output to send responses back
    /// mailbox allows the output to read the message
    fn spawn_next(&self, index: usize, metadata: MetaData, address: Option<Self::Address>, mailbox: Option<Self::Mailbox>);

    /// establish a new channel
    fn new_channel(&self) -> (Self::Address, Self::Mailbox);

    /// spawn a task that runs on the background
    /// no handler is returned. Use channels to get results if necessary.
    fn spawn_task(&self, task: impl Future + Send + 'static);
}

/// Meta data dict.
/// Cloning a MetaData will be "shallow". However, the values in MetaData are immutable unless it has interior mutability.
#[derive(Default, Debug, Clone)]
pub struct MetaData(BTreeMap<String, Arc<Box<dyn Any + Send + Sync>>>);

impl MetaData {
    /// Get a value in the meta data. Return None if the key does not exist or the type mismatches.
    pub fn get<T: 'static>(&self, key: &str) -> Option<&T> {
        self.0.get(key)?.downcast_ref()
    }

    /// Set a value in the meta data. Old value is dropped if the key already exists.
    pub fn set<T: Any + Send + Sync>(&mut self, key: String, value: T) {
        self.0.insert(key, Arc::new(Box::new(value)));
    }

    /// Take out a value. Remove the key in any case.
    /// If the type mismatches or the value is borrowed elsewhere, None is returned.
    pub fn take<T: 'static>(&mut self, key: &str) -> Option<Box<T>> {
        Arc::try_unwrap(self.0.remove(key)?).ok()?.downcast().ok()
    }
}

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
    /// direction (String): "forward" or "backward"
    /// outputs (List<String>): the names of outputs. Unamed outputs have empty names.
    fn create(&'static self, arguments: Vec<(String, Argument)>) -> Result<Box<dyn Actor<R>>>;
}


