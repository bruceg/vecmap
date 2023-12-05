use std::collections::{BTreeMap, HashMap};

use vecmap::VecMap;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

type Key = String;
type Value = u64;

#[derive(Clone)]
enum Action {
    Insert(Key, Value),
    Remove(Key),
    Get(Key),
}

impl Action {
    fn random(rng: &mut fastrand::Rng) -> Self {
        let key: String = (0..rng.usize(1..15)).map(|_| rng.alphanumeric()).collect();
        match rng.usize(0..200) {
            0..=15 => Self::Insert(key, rng.u64(..)),
            16..=31 => Self::Remove(key),
            _ => Self::Get(key),
        }
    }
}

trait Mapper: Default {
    fn do_insert(&mut self, key: Key, value: Value) -> Option<Value>;
    fn do_remove(&mut self, key: &Key) -> Option<Value>;
    fn do_get(&mut self, key: &Key) -> Option<&Value>;
}

impl Mapper for BTreeMap<Key, Value> {
    fn do_insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.insert(key, value)
    }
    fn do_remove(&mut self, key: &Key) -> Option<Value> {
        self.remove(key)
    }
    fn do_get(&mut self, key: &Key) -> Option<&Value> {
        self.get(key)
    }
}

impl Mapper for HashMap<Key, Value> {
    fn do_insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.insert(key, value)
    }
    fn do_remove(&mut self, key: &Key) -> Option<Value> {
        self.remove(key)
    }
    fn do_get(&mut self, key: &Key) -> Option<&Value> {
        self.get(key)
    }
}

impl Mapper for VecMap<Key, Value> {
    fn do_insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.insert(key, value)
    }
    fn do_remove(&mut self, key: &Key) -> Option<Value> {
        self.remove(key)
    }
    fn do_get(&mut self, key: &Key) -> Option<&Value> {
        self.get(key)
    }
}

#[divan::bench(
    types = [BTreeMap<Key, Value>, HashMap<Key, Value>, VecMap<Key, Value>],
    consts = [0, 8, 64, 1024],
)]
fn mapit<T: Mapper, const L: usize>(bencher: divan::Bencher) {
    bencher
        .counter(L)
        .with_inputs(|| {
            let mut rng = fastrand::Rng::new();
            (0..L).map(|_| Action::random(&mut rng)).collect()
        })
        .bench_values(|actions: Vec<Action>| {
            let mut map = T::default();
            for action in actions {
                match action {
                    Action::Insert(key, value) => {
                        map.do_insert(key, value);
                    }
                    Action::Remove(key) => {
                        map.do_remove(&key);
                    }
                    Action::Get(key) => {
                        map.do_get(&key);
                    }
                }
            }
        });
}
