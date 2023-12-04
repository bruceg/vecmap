use std::collections::BTreeMap;
use std::fmt::Debug;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct VecMap<K: Ord, V>(BTreeMap<K, V>);

impl<K: Ord, V> VecMap<K, V> {
    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    pub fn iter(&self) -> <&BTreeMap<K, V> as IntoIterator>::IntoIter {
        self.0.iter()
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.0.insert(key, value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }
}

impl<K: Ord, V> IntoIterator for VecMap<K, V> {
    type Item = (K, V);
    type IntoIter = <BTreeMap<K, V> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for VecMap<K, V> {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest_derive::Arbitrary;

    use super::*;

    #[derive(Arbitrary, Debug)]
    enum Action<K, V> {
        Get(K),
        Insert(K, V),
        Remove(K),
    }

    fn compare_map<K: Debug + Eq + Ord, V: Debug + Eq>(this: &BTreeMap<K, V>, that: &VecMap<K, V>) {
        assert_eq!(
            this.iter().collect::<Vec<_>>(),
            that.iter().collect::<Vec<_>>()
        );
    }

    fn run_tests<K, V>(start: Vec<(K, V)>, acts: &[Action<K, V>])
    where
        K: Clone + Debug + Eq + Ord,
        V: Clone + Debug + Eq,
    {
        let mut baseline = BTreeMap::from_iter(start.clone());
        let mut sut = VecMap::from_iter(start);

        compare_map(&baseline, &sut);

        for act in acts {
            match act {
                Action::Get(key) => {
                    assert_eq!(baseline.get(key), sut.get(key));
                }
                Action::Insert(key, value) => {
                    assert_eq!(
                        baseline.insert(key.clone(), value.clone()),
                        sut.insert(key.clone(), value.clone()),
                    );
                }
                Action::Remove(key) => {
                    assert_eq!(baseline.remove(key), sut.remove(key));
                }
            }
            compare_map(&baseline, &sut);
        }
    }

    proptest! {
        /// This uses `u8` for the key type simply to reduce the key
        /// space and ensure more same-key interactions happen.
        #[test]
        fn behaves_like_btreemap(
            start in any::<Vec<(u8, i64)>>(),
            acts in any::<Vec<Action<u8, i64>>>()
        ) {
            run_tests(start, &acts);
        }
    }
}
