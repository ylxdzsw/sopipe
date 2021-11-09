use std::{collections::BTreeMap, error::Error, future::Future, pin::Pin};
pub use async_trait::async_trait; // expose to components

pub mod helper; // helper lib for components

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub type Actor = dyn (Fn(Box<dyn Runtime>, BTreeMap<String, ArgumentValue>) -> Result<Pin<Box<dyn Future<Output=Result<()>> + Send>>>) + Sync;

#[derive(Debug, Clone)]
pub struct Argument(pub String, pub ArgumentValue);

#[derive(Debug, Clone)]
pub enum ArgumentValue {
    String(String),
    Int(u64),
    Vec(Vec<ArgumentValue>),
    None
}

impl From<String> for ArgumentValue {
    fn from(x: String) -> Self {
        Self::String(x)
    }
}

impl From<u64> for ArgumentValue {
    fn from(x: u64) -> Self {
        Self::Int(x)
    }
}

impl<T> FromIterator<T> for ArgumentValue where ArgumentValue: std::convert::From<T> {
    fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
        Self::Vec(iter.into_iter().map(ArgumentValue::from).collect())
    }
}

impl ArgumentValue {
    pub fn type_name(&self) -> &'static str {
        match &self {
            ArgumentValue::String(_) => "string",
            ArgumentValue::Int(_) => "int",
            ArgumentValue::Vec(_) => "vec",
            ArgumentValue::None => "none",
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        match &self {
            &ArgumentValue::String(x) => Some(x),
            _ => None
        }
    }

    pub fn as_int(&self) -> Option<&u64> {
        match &self {
            &ArgumentValue::Int(x) => Some(x),
            _ => None
        }
    }

    pub fn as_vec(&self) -> Option<&[ArgumentValue]> {
        match &self {
            &ArgumentValue::Vec(x) => Some(x),
            _ => None
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, &ArgumentValue::None)
    }
}

#[async_trait]
pub trait Address: Sync + Send {
    async fn send(&self, msg: Box<[u8]>);
}

#[async_trait]
pub trait Runtime: Sync + Send {
    /// get a buffer
    async fn read(&mut self) -> Option<Box<[u8]>>;

    /// spawn an actor of the i-th output with args about the stream, return its address
    fn spawn(&self, index: usize, metadata: BTreeMap<String, ArgumentValue>) -> Box<dyn Address>;

    /// spawn another actor of this node with args about the stream, return its address
    fn spawn_self(&self, metadata: BTreeMap<String, ArgumentValue>) -> Box<dyn Address>;

    /// spawn an actor of the conjugate node with args about the stream, return its address
    fn spawn_conjugate(&self, metadata: BTreeMap<String, ArgumentValue>) -> Box<dyn Address>;

    /// indicate if this actor is a source node (no input)
    fn is_source(&self) -> bool;
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
    fn create(&'static self, arguments: Vec<Argument>) -> Result<Box<Actor>>;
}


