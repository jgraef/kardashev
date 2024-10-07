use std::{
    any::{
        Any,
        TypeId,
    },
    collections::{
        hash_map,
        HashMap,
    },
    convert::Infallible,
    fmt::Debug,
    future::Future,
    hash::Hash,
    sync::{
        Arc,
        Weak,
    },
};

pub struct AnyArcCache<K> {
    cache: HashMap<(K, TypeId), Weak<dyn Any + Send + Sync + 'static>>,
}

impl<K: Eq + Hash> AnyArcCache<K> {
    pub fn remove_stale(&mut self) {
        self.cache.retain(|_, weak| weak.strong_count() > 0);
    }

    pub fn get<T>(&self, key: K) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.cache
            .get(&(key, TypeId::of::<T>()))
            .and_then(|weak| weak.upgrade())
            .map(|strong| Arc::downcast::<T>(strong).expect("downcast failed"))
    }

    pub fn insert<T>(&mut self, key: K, data: &Arc<T>)
    where
        T: Send + Sync + 'static,
    {
        let weak = Arc::downgrade(data);
        self.cache.insert((key, TypeId::of::<T>()), weak);
    }

    pub fn get_or_insert<T, F>(&mut self, key: K, f: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Arc<T>,
    {
        self.get_or_try_insert(key, || Ok::<_, Infallible>(f()))
            .unwrap()
    }

    pub fn get_or_try_insert<T, F, E>(&mut self, key: K, f: F) -> Result<Arc<T>, E>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Result<Arc<T>, E>,
    {
        match self.cache.entry((key, TypeId::of::<T>())) {
            hash_map::Entry::Occupied(mut occupied) => {
                if let Some(strong) = occupied.get().upgrade() {
                    Ok(Arc::downcast::<T>(strong).expect("downcast failed"))
                }
                else {
                    let data = f()?;
                    let weak = Arc::downgrade(&data);
                    occupied.insert(weak);
                    Ok(data)
                }
            }
            hash_map::Entry::Vacant(vacant) => {
                let data = f()?;
                let weak = Arc::downgrade(&data);
                vacant.insert(weak);
                Ok(data)
            }
        }
    }

    pub async fn get_or_try_insert_async<T, F, Fut, E>(&mut self, key: K, f: F) -> Result<Arc<T>, E>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Arc<T>, E>>,
    {
        match self.cache.entry((key, TypeId::of::<T>())) {
            hash_map::Entry::Occupied(mut occupied) => {
                if let Some(strong) = occupied.get().upgrade() {
                    Ok(Arc::downcast::<T>(strong).expect("downcast failed"))
                }
                else {
                    let data = f().await?;
                    let weak = Arc::downgrade(&data);
                    occupied.insert(weak);
                    Ok(data)
                }
            }
            hash_map::Entry::Vacant(vacant) => {
                let data = f().await?;
                let weak = Arc::downgrade(&data);
                vacant.insert(weak);
                Ok(data)
            }
        }
    }
}

impl<K> Default for AnyArcCache<K> {
    fn default() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl<K> Debug for AnyArcCache<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyArcCache").finish_non_exhaustive()
    }
}
