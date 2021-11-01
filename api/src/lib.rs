use std::{error::Error, ptr::NonNull};
pub use async_trait::async_trait; // expose to components

#[derive(Debug, Clone)]
pub struct Argument(pub String, pub ArgumentValue);

#[derive(Debug, Clone)]
pub enum ArgumentValue {
    String(String),
    Int(u64),
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

impl ArgumentValue {
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
}

#[async_trait]
pub trait Actor: Send {
    async fn feed(&mut self, );
}

pub trait Component: Sync {
    /// get the name of functions this component registers
    fn functions(&self) -> &'static [&'static str];

    /// create an instance for a node in the pipeline.
    /// the arguments includes user-provided arguments as well as the following:
    /// function_name (String): the name of function in the user script
    /// direction (String): "forward" or "backward"
    /// n_outputs (Int): the number of outputs
    fn create(&self, arguments: Vec<Argument>) -> Result<NonNull<()>, Box<dyn Error + Send + Sync>>;

    /// spawn an actor
    fn spawn(&self, node_state: *const ()) -> Result<Box<dyn Actor>, Box<dyn Error + Send + Sync>>;
}


