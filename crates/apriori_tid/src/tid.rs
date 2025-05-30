use apriori::{
    apriori::AprioriP1,
    start::{AprioriOne, Write},
    storage::{AprioriCounter, AprioriFrequent},
    transaction_set::TransactionSet,
    trie::{TrieCounter, TrieSet},
};

pub struct AprioriTIDRunner<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl<'a> AprioriTIDRunner<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
    pub fn run<T: Write>(self, writer: &mut T) {
        let mut ids = TransactionIDs::<TrieSet>::new(self.data);
        let mut p: TrieSet = AprioriP1::new(self.data, self.sup).run_one();
        for n in 2.. {
            p.for_each(|v| {
                if v.len() != n - 1 {
                    return;
                }
                writer.write_set(v);
            });
            let p2: TrieCounter = ids.run(&mut p);
            p = p2.to_frequent_new(self.sup);
            if p.is_empty() {
                break;
            }
        }
    }
}
pub struct TransactionIDs<T: TransactionID> {
    ids: Vec<T>,
}

impl<T: TransactionID + Default> TransactionIDs<T> {
    pub fn new(d: &TransactionSet) -> Self {
        let mut ids = Vec::new();
        for d in d.iter() {
            ids.push(T::from_data(d));
        }
        Self { ids }
    }
    pub fn run<U: AprioriFrequent, V: AprioriCounter + Default>(&mut self, frequent: &mut U) -> V {
        let mut counter = frequent.join_new();
        self.count(&mut counter);
        counter
    }
    pub fn count<U: AprioriCounter>(&mut self, counter: &mut U) {
        for id in self.ids.iter_mut() {
            *id = id.count_new(counter);
        }
    }
}

pub trait TIDCount {
    fn count<T: TransactionID, U: AprioriCounter>(&self, new: &mut T, counter: &mut U);
    fn count_new<T: TransactionID + Default, U: AprioriCounter>(&self, counter: &mut U) -> T {
        let mut new = T::default();
        self.count(&mut new, counter);
        new
    }
}

impl<A: TransactionID> TIDCount for A {
    fn count<T: TransactionID, U: AprioriCounter>(&self, new: &mut T, counter: &mut U) {
        self.join_fn(|v| {
            if counter.increment(v) {
                new.insert(v);
            }
        });
    }
}

pub trait TransactionID: AprioriFrequent {
    fn from_data(data: &[usize]) -> Self;
}
impl TransactionID for TrieSet {
    fn from_data(data: &[usize]) -> Self {
        let mut s = Self::new();
        for &n in data {
            s.insert(&[n]);
        }
        s
    }
}
