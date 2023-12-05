use std::collections::HashMap;
use std::hash::Hash;
use std::iter;
use std::marker::PhantomData;
use std::sync::Arc;

use dashmap::DashMap;

////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
struct KeysInner<K: Eq + Hash> {
    indices: HashMap<K, usize>,
    inserts: DashMap<K, SharedKeys<K>>,
}

impl<K: Clone + Eq + Hash> KeysInner<K> {
    fn insert(&self, key: K) -> SharedKeys<K> {
        // Look for an existing insert chain and return it
        if let Some(next) = self.inserts.get(&key) {
            return next.clone();
        }

        // If it wasn't already present (the common case), add a new
        // insert entry with the new key added to the end of the key
        // indices.
        self.inserts
            .entry(key.clone())
            .or_insert_with(|| {
                let index = self
                    .indices
                    .values()
                    .copied()
                    .max()
                    .map(|index| index + 1)
                    .unwrap_or(0);
                SharedKeys::from_iter(
                    self.indices
                        .iter()
                        .map(|(key, &index)| (key.clone(), index))
                        .chain(iter::once((key, index))),
                )
            })
            .value()
            .clone()
    }
}

impl<K: Eq + Hash> FromIterator<K> for KeysInner<K> {
    fn from_iter<I: IntoIterator<Item = K>>(keys: I) -> Self {
        Self::from_iter(
            keys.into_iter()
                .enumerate()
                .map(|(index, key)| (key, index)),
        )
    }
}

impl<K: Eq + Hash> FromIterator<(K, usize)> for KeysInner<K> {
    fn from_iter<I: IntoIterator<Item = (K, usize)>>(indices: I) -> Self {
        Self {
            indices: indices.into_iter().collect(),
            inserts: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct SharedKeys<K: Eq + Hash>(Arc<KeysInner<K>>);

impl<K: Eq + Hash> SharedKeys<K> {
    pub fn get_index(&self, key: &K) -> Option<usize> {
        self.0.indices.get(key).copied()
    }
}

impl<K: Eq + Hash> SharedKeys<K> {
    pub fn indices(&self) -> &HashMap<K, usize> {
        &self.0.indices
    }
}

impl<K: Clone + Eq + Hash> SharedKeys<K> {
    pub fn insert(&self, key: K) -> Self {
        self.0.insert(key)
    }
}

impl<K: Eq + Hash> Clone for SharedKeys<K> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<K: Eq + Hash> FromIterator<K> for SharedKeys<K> {
    fn from_iter<I: IntoIterator<Item = K>>(keys: I) -> Self {
        Self(Arc::new(KeysInner::from_iter(keys)))
    }
}

impl<K: Eq + Hash> FromIterator<(K, usize)> for SharedKeys<K> {
    fn from_iter<I: IntoIterator<Item = (K, usize)>>(indices: I) -> Self {
        Self(Arc::new(KeysInner::from_iter(indices)))
    }
}

////////////////////////////////////////////////////////////////////////
pub trait KeyStore<K: Eq + Hash> {
    fn get(keys: Vec<K>) -> SharedKeys<K>;
}

////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct StdKeyStore<K>(PhantomData<K>);

impl<K: Eq + Hash + Send + Sync> StdKeyStore<K> {
    pub fn get_map() -> &'static DashMap<Vec<K>, SharedKeys<K>> {
        generic_singleton::get_or_init!(DashMap::new)
    }
}

impl<K: Clone + Eq + Hash + Send + Sync + 'static> KeyStore<K> for StdKeyStore<K> {
    fn get(keys: Vec<K>) -> SharedKeys<K> {
        let map = Self::get_map();
        map.get(&keys)
            .map(|got| got.value().clone())
            .unwrap_or_else(|| {
                let value = SharedKeys::from_iter(keys.iter().cloned());
                let entry = map.entry(keys).or_insert(value);
                entry.value().clone()
            })
    }
}
