use std::collections::{BTreeMap, HashMap};
use std::iter;
use std::sync::OnceLock;

use criterion::{
    black_box, criterion_group, criterion_main, measurement::WallTime, BatchSize, BenchmarkGroup,
    BenchmarkId, Criterion, Throughput,
};
use fastrand::Rng;

use vecmap::NewMap;

const SIZES: [usize; 12] = [0, 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
const MAX_KEYS: usize = 8;
const MAX_KEY_SIZE: usize = 30;

type Key = String;
type Value = u64;

fn random_key(rng: &mut Rng) -> Key {
    static KEYS: OnceLock<Vec<String>> = OnceLock::new();
    let keys = KEYS
        .get_or_init(|| {
            (0..MAX_KEYS)
                .map(|i| {
                    let mut rng1 = rng.clone();
                    format!(
                        "{i} {}",
                        iter::repeat_with(|| rng1.alphanumeric())
                            .take(rng.usize(1..=MAX_KEY_SIZE))
                            .collect::<String>()
                    )
                })
                .collect()
        })
        .as_slice();
    keys[rng.usize(0..keys.len())].clone()
}

#[derive(Clone)]
enum Action {
    Insert(Key, Value),
    Remove(Key),
    Get(Key),
}

impl Action {
    fn random(rng: &mut Rng) -> Self {
        let key = random_key(rng);
        match rng.usize(0..100) {
            0..=9 => Self::Insert(key, rng.u64(..)),
            10..=19 => Self::Remove(key),
            _ => Self::Get(key),
        }
    }
}

trait Mapper: Default + FromIterator<(Key, Value)> {
    fn run_actions(init: Vec<(Key, Value)>, actions: Vec<Action>) {
        let mut map = Self::from_iter(init);
        for action in actions {
            map.do_action(action);
        }
    }
    fn do_action(&mut self, action: Action) {
        match action {
            Action::Insert(key, value) => {
                self.do_insert(key, value);
            }
            Action::Remove(key) => {
                self.do_remove(&key);
            }
            Action::Get(key) => {
                black_box(self.do_get(&key));
            }
        }
    }
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

impl Mapper for NewMap<Key, Value> {
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

fn make_init(rng: &mut Rng) -> Vec<(Key, Value)> {
    static KEY_SETS: OnceLock<Vec<Vec<(Key, Value)>>> = OnceLock::new();
    let inits = KEY_SETS.get_or_init(|| {
        (0..MAX_KEYS * MAX_KEYS)
            .map(|_| {
                (0..rng.usize(0..MAX_KEYS))
                    .map(|_| (random_key(rng), rng.u64(..)))
                    .collect()
            })
            .collect()
    });
    inits[rng.usize(0..inits.len())].clone()
}

fn make_actions(size: usize, rng: &mut Rng) -> Vec<Action> {
    iter::repeat_with(|| Action::random(rng))
        .take(size)
        .collect::<Vec<_>>()
}

fn run_map<T: Mapper>(id: &str, group: &mut BenchmarkGroup<'_, WallTime>, size: usize) {
    let mut rng = Rng::new();
    group.bench_with_input(BenchmarkId::new(id, size), &size, |b, &size| {
        b.iter_batched(
            || (make_init(&mut rng), make_actions(size, &mut rng)),
            |(init, batch)| T::run_actions(init, batch),
            BatchSize::LargeInput,
        );
    });
}

fn bench_maps(c: &mut Criterion) {
    let mut group = c.benchmark_group("map random acts");
    for size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        run_map::<HashMap<Key, Value>>("HashMap", &mut group, size);
        run_map::<BTreeMap<Key, Value>>("BTreeMap", &mut group, size);
        run_map::<NewMap<Key, Value>>("NewMap", &mut group, size);
    }
}

criterion_group!(benches, bench_maps);
criterion_main!(benches);
