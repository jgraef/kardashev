use std::{
    any::{
        Any,
        TypeId,
    },
    collections::{
        hash_map,
        HashMap,
    },
    fmt::Debug,
    future::Future,
    hash::Hash,
};

pub struct AnyCache<K> {
    cache: HashMap<(K, TypeId), Box<dyn Any + Send + Sync + 'static>>,
}

impl<K: Eq + Hash> AnyCache<K> {
    pub fn get<T>(&self, key: K) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.cache
            .get(&(key, TypeId::of::<T>()))
            .map(|data| (**data).downcast_ref::<T>().unwrap().clone())
    }

    pub fn insert<T>(&mut self, key: K, data: T)
    where
        T: Send + Sync + 'static,
    {
        self.cache.insert((key, TypeId::of::<T>()), Box::new(data));
    }

    pub fn get_or_try_insert<T, F, E>(&mut self, key: K, f: F) -> Result<T, E>
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> Result<T, E>,
    {
        match self.cache.entry((key, TypeId::of::<T>())) {
            hash_map::Entry::Occupied(occupied) => {
                Ok((**occupied.get()).downcast_ref::<T>().unwrap().clone())
            }
            hash_map::Entry::Vacant(vacant) => {
                let data = f()?;
                vacant.insert(Box::new(data.clone()));
                Ok(data)
            }
        }
    }

    pub async fn get_or_try_insert_async<T, F, Fut, E>(&mut self, key: K, f: F) -> Result<T, E>
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        match self.cache.entry((key, TypeId::of::<T>())) {
            hash_map::Entry::Occupied(occupied) => {
                Ok((**occupied.get()).downcast_ref::<T>().unwrap().clone())
            }
            hash_map::Entry::Vacant(vacant) => {
                let data = f().await?;
                vacant.insert(Box::new(data.clone()));
                Ok(data)
            }
        }
    }
}

impl<K> Default for AnyCache<K> {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<K> Debug for AnyCache<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cache").finish_non_exhaustive()
    }
}
