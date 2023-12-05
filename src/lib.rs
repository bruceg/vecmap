mod keystore;
mod map;

pub use map::NewMap;

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fmt::Debug;
    use std::hash::Hash;

    use proptest::prelude::*;
    use proptest_derive::Arbitrary;

    use super::*;

    #[test]
    fn basic() {
        let mut map = NewMap::<u8, u64>::default();
        assert_eq!(map.len(), 0);

        assert_eq!(map.insert(0, 100), None);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&0), Some(&100));

        assert_eq!(map.insert(1, 101), None);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&0), Some(&100));
        assert_eq!(map.get(&1), Some(&101));

        assert_eq!(map.insert(1, 102), Some(101));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&0), Some(&100));
        assert_eq!(map.get(&1), Some(&102));

        assert_eq!(map.remove(&0), Some(100));
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&0), None);
        assert_eq!(map.get(&1), Some(&102));
    }

    #[derive(Arbitrary, Debug)]
    enum Action<K, V> {
        Get(K),
        Insert(K, V),
        Remove(K),
    }

    fn compare_map<K, V>(this: &BTreeMap<K, V>, that: &NewMap<K, V>)
    where
        K: Clone + Debug + Eq + Hash + Ord + Send + Sync + 'static,
        V: Debug + Eq + Ord,
    {
        let mut this: Vec<_> = this.iter().collect();
        let mut that: Vec<_> = that.iter().collect();
        this.sort();
        that.sort();
        assert_eq!(this, that);
    }

    fn run_tests<K, V>(start: Vec<(K, V)>, acts: &[Action<K, V>])
    where
        K: Clone + Debug + Eq + Hash + Ord + Send + Sync + 'static,
        V: Clone + Debug + Eq + Ord,
    {
        let mut baseline = BTreeMap::from_iter(start.clone());
        let mut sut = NewMap::from_iter(start);

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
