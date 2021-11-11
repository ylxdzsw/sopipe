use std::{any::Any, collections::BTreeMap, error::Error, future::Future, pin::Pin, sync::Arc};
pub use async_trait::async_trait; // expose to components
pub use tokio; // expose to components
pub mod helper; // helper lib for components

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
pub type Address = tokio::sync::mpsc::Sender<Box<[u8]>>;
pub type Mailbox = tokio::sync::mpsc::Receiver<Box<[u8]>>;

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

/// A trait that provides runtime functions to components. It is tied to each actor.
pub trait Runtime: Sync + Send {
    /// spawn an actor of the i-th output
    /// metadata provides information about this stream
    /// address allows the output to send responses back
    /// mailbox allows the output to read the message
    fn spawn_next(&self, index: usize, metadata: MetaData, address: Option<Address>, mailbox: Option<Mailbox>);
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

pub trait Actor: Sync {
    /// spawn an instance of this actor, handling messages in the mailbox and send responses to the address.
    fn spawn(&'static self, runtime: Box<dyn Runtime>, metadata: MetaData, address: Option<Address>, mailbox: Option<Mailbox>);

    /// spawn an instance of this actor as a part in a composited component. It acts like a one-way pipe that process messages from the mailbox and send to the address.
    fn spawn_composite(&'static self, runtime: Box<dyn Runtime>, metadata: MetaData, address: Option<Address>, mailbox: Option<Mailbox>);

    /// spawn an instance of this actor as a source node
    fn spawn_source(&'static self, runtime: Box<dyn Runtime>);
}

/// The main trait for components.
pub trait Component: Sync {
    /// get the name of functions this component registers
    fn functions(&'static self) -> &'static [&'static str];

    /// create an instance for a node in the pipeline.
    /// the arguments includes user-provided arguments as well as the following:
    /// function_name (String): the name of function in the user script
    /// direction (String): "forward" or "backward"
    /// outputs (List<String>): the names of outputs. Unamed outputs have empty names.
    fn create(&'static self, arguments: Vec<(String, Argument)>) -> Result<Box<dyn Actor>>;
}


