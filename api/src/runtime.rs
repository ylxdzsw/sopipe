use std::{future::Future, pin::Pin};

use super::MetaData;

pub trait Address: Clone + Send + Sync {
    fn send(&mut self, msg: Box<[u8]>) -> Pin<Box<dyn Future<Output=Result<(), ()>> + Send + '_>>;
}

pub trait Mailbox: Send + Sync {
    #[allow(clippy::type_complexity)]
    fn recv(&mut self) -> Pin<Box<dyn Future<Output=Option<Box<[u8]>>> + Send + '_>>;
}

/// A trait that provides runtime functions to components. It is tied to each actor.
pub trait Runtime: Sync + Send {
    type Address: Address + 'static;
    type Mailbox: Mailbox + 'static;

    /// spawn an actor of the i-th output
    /// metadata provides information about this stream
    /// address allows the output to send responses back
    /// mailbox allows the output to read the message
    fn spawn_next(&self, index: usize, metadata: MetaData, address: impl Into<Option<Self::Address>>, mailbox: impl Into<Option<Self::Mailbox>>);

    /// establish a new channel
    fn channel(&self) -> (Self::Address, Self::Mailbox);

    /// spawn a task that runs on the background
    /// no handler is returned. Use channels to get results if necessary.
    fn spawn_task(&self, task: impl Future + Send + 'static);
}
