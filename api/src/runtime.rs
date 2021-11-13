use std::{future::Future, pin::Pin};

use super::MetaData;

#[derive(Clone, Copy)]
pub enum RunLevel { Init, Run, Shut }

// TODO: Box<[u8]> causes a lot of allocation and memcpy. Design a structure that can grow on both sides? Ideally components can give hints about how many bytes they are going to add, so we can preallocate at the begining.

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
    fn spawn_task<F: Future + Send + 'static>(&self, task: F) where F::Output: Send;

    /// get the current runlevel. Only source nodes need to care about this.
    fn get_runlevel(&self) -> RunLevel;

    /// watch changes on the runlevel.
    fn watch_runlevel(&self) -> Pin<Box<dyn Future<Output=()> + Send + '_>>;
}
