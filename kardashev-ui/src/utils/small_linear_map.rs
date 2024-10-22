use std::fmt::Debug;

use smallvec::SmallVec;

#[derive(Clone, Debug)]
struct Item<K, V> {
    key: K,
    value: V,
}

#[derive(Clone)]
pub struct SmallLinearMap<const N: usize, K, V> {
    items: SmallVec<[Item<K, V>; N]>,
}

impl<const N: usize, K, V> Default for SmallLinearMap<N, K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, K, V> SmallLinearMap<N, K, V> {
    pub const fn new() -> Self {
        Self {
            items: SmallVec::new_const(),
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            inner: self.items.iter(),
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }
}

impl<const N: usize, K: Eq, V> SmallLinearMap<N, K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.items
            .iter()
            .find(|item| &item.key == key)
            .map(|item| &item.value)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.items
            .iter_mut()
            .find(|item| &item.key == key)
            .map(|item| &mut item.value)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.items.iter().find(|item| &item.key == key).is_some()
    }

    pub fn entry<'a>(&'a mut self, key: K) -> Entry<'a, N, K, V> {
        if let Some((index, _)) = self
            .items
            .iter_mut()
            .enumerate()
            .find(|(_, item)| &item.key == &key)
        {
            Entry::Occupied(OccupiedEntry {
                items: &mut self.items,
                index,
            })
        }
        else {
            Entry::Vacant(VacantEntry {
                items: &mut self.items,
                key,
            })
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entry(key).insert_entry(value);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let (index, _) = self
            .items
            .iter()
            .enumerate()
            .find(|(_, item)| &item.key == key)?;
        let item = self.items.swap_remove(index);
        Some(item.value)
    }
}

impl<const N: usize, K: Debug, V: Debug> Debug for SmallLinearMap<N, K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

pub enum Entry<'a, const N: usize, K, V> {
    Occupied(OccupiedEntry<'a, N, K, V>),
    Vacant(VacantEntry<'a, N, K, V>),
}

impl<'a, const N: usize, K, V> Entry<'a, N, K, V> {
    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        match &mut self {
            Self::Occupied(occupied) => f(occupied.get_mut()),
            Self::Vacant(_) => {}
        }
        self
    }

    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, N, K, V> {
        match self {
            Self::Occupied(mut occupied) => {
                *occupied.get_mut() = value;
                occupied
            }
            Self::Vacant(vacant) => vacant.insert_entry(value),
        }
    }

    pub fn key(&self) -> &K {
        match self {
            Self::Occupied(occupied) => occupied.key(),
            Self::Vacant(vacant) => vacant.key(),
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        self.or_insert_with(move || default)
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        match self {
            Self::Occupied(occupied) => occupied.into_mut(),
            Self::Vacant(vacant) => vacant.insert_entry(f()).into_mut(),
        }
    }
}

impl<'a, const N: usize, K, V: Default> Entry<'a, N, K, V> {
    pub fn or_default(self) -> &'a mut V {
        self.or_insert_with(Default::default)
    }
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, const N: usize, K, V> {
    items: &'a mut SmallVec<[Item<K, V>; N]>,
    index: usize,
}

impl<'a, const N: usize, K, V> OccupiedEntry<'a, N, K, V> {
    pub fn get(&self) -> &V {
        &self.items[self.index].value
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.items[self.index].value
    }

    pub fn into_mut(self) -> &'a mut V {
        &mut self.items[self.index].value
    }

    pub fn key(&self) -> &K {
        &self.items[self.index].key
    }

    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(&mut self.items[self.index].value, value)
    }

    pub fn remove(self) -> V {
        self.items.swap_remove(self.index).value
    }

    pub fn remove_entry(self) -> (K, V) {
        let item = self.items.swap_remove(self.index);
        (item.key, item.value)
    }
}

#[derive(Debug)]
pub struct VacantEntry<'a, const N: usize, K, V> {
    items: &'a mut SmallVec<[Item<K, V>; N]>,
    key: K,
}

impl<'a, const N: usize, K, V> VacantEntry<'a, N, K, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        let index = self.items.len();
        self.items.push(Item {
            key: self.key,
            value,
        });
        &mut self.items[index].value
    }

    pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, N, K, V> {
        let index = self.items.len();
        self.items.push(Item {
            key: self.key,
            value,
        });
        OccupiedEntry {
            items: self.items,
            index,
        }
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

pub struct Iter<'a, K, V> {
    inner: std::slice::Iter<'a, Item<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        Some((&item.key, &item.value))
    }
}
