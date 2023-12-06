use std::collections::hash_map;
use std::hash::Hash;
use std::marker::PhantomData;

use itertools::Itertools as _;

use super::keystore::{KeyStore, SharedKeys, StdKeyStore};

pub struct NewMap<K: Eq + Hash, V, S = StdKeyStore<K>> {
    keys: SharedKeys<K>,
    values: Vec<Option<V>>,
    count: usize,
    _store: PhantomData<S>,
}

impl<K: Eq + Hash, V, S: KeyStore<K>> Default for NewMap<K, V, S> {
    fn default() -> Self {
        let keys = S::get(Vec::new());
        Self {
            keys,
            values: Vec::new(),
            count: 0,
            _store: Default::default(),
        }
    }
}

impl<K: Eq + Hash, V, S> NewMap<K, V, S> {
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl<K: Eq + Hash, V, S: KeyStore<K>> NewMap<K, V, S> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K: Eq + Hash, V, S: KeyStore<K>> NewMap<K, V, S> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys
            .get_index(key)
            .and_then(|i| self.values[i].as_ref())
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        self.into_iter()
    }
}

impl<K: Clone + Eq + Hash + Ord, V, S> NewMap<K, V, S> {
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.keys.indices().get(&key) {
            Some(&index) => self.values[index].replace(value).or_else(|| {
                self.count += 1;
                None
            }),
            None => {
                self.keys = self.keys.insert(key);
                self.values.push(Some(value));
                self.count += 1;
                None
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.keys
            .indices()
            .get(key)
            .and_then(|&index| self.values[index].take())
            .map(|value| {
                self.count -= 1;
                value
            })
    }
}

impl<K: Eq + Hash + Ord, V, S: KeyStore<K>> FromIterator<(K, V)> for NewMap<K, V, S> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let (keys, values): (Vec<_>, Vec<_>) = iter
            .into_iter()
            .map(|(key, value)| (key, Some(value)))
            .multiunzip();
        let keys = S::get(keys);
        Self {
            count: values.len(),
            keys,
            values,
            _store: Default::default(),
        }
    }
}

impl<'a, K: Eq + Hash, V, S> IntoIterator for &'a NewMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            keys: self.keys.indices().iter(),
            values: &self.values,
        }
    }
}

pub struct Iter<'a, K: Eq + Hash, V> {
    keys: hash_map::Iter<'a, K, usize>,
    values: &'a Vec<Option<V>>,
}

impl<'a, K: Eq + Hash, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        for (key, &index) in self.keys.by_ref() {
            if let Some(value) = self.values[index].as_ref() {
                return Some((key, value));
            }
        }
        None
    }
}

impl<K: Clone + Eq + Hash, V, S> IntoIterator for NewMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            keys: self.keys.indices().clone().into_iter(),
            values: self.values,
        }
    }
}

pub struct IntoIter<K, V> {
    keys: hash_map::IntoIter<K, usize>,
    values: Vec<Option<V>>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        for (key, index) in self.keys.by_ref() {
            if let Some(value) = self.values[index].take() {
                return Some((key, value));
            }
        }
        None
    }
}
