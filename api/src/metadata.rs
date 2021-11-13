use std::{any::Any, collections::BTreeMap, sync::Arc};

// TODO: what about define a struct that keeps all predefined keys with types?

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
