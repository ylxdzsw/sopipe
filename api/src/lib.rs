use std::{error::Error, ptr::NonNull};
pub use async_trait::async_trait; // expose to components

#[derive(Debug, Clone)]
pub struct Argument(pub String, pub ArgumentValue);

#[derive(Debug, Clone)]
pub enum ArgumentValue {
    String(String),
    Int(u64),
}

#[async_trait]
pub trait Actor: Send {
    async fn feed(&mut self, );
}

pub trait Component: Sync {
    /// get the name of functions this component registers
    fn functions(&self) -> &'static [&'static str];

    fn create(&self) -> Result<NonNull<()>, Box<dyn Error>>;

}


