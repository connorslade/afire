use std::{any::Any, collections::HashMap, hash::Hash, sync::RwLock};

pub struct Store<T> {
    inner: RwLock<HashMap<T, Box<dyn Any>>>,
}

impl<T: Eq + Hash> Store<T> {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&self, key: T, value: impl Any) {
        self.inner.write().unwrap().insert(key, Box::new(value));
    }

    pub fn get<K>(&self, key: &T) -> impl Any
    where
        Self: 'static,
    {
        // self.inner.read().unwrap().get(key)
        todo!()
    }
}
