use apriori::{
    apriori::{AprioriP1, AprioriP2},
    start::{AprioriOne, AprioriTwo, Write},
    storage::{AprioriCounter, AprioriCounterMut, AprioriCounting, AprioriFrequent},
    transaction_set::TransactionSet,
    trie::{TrieCounter, TrieSet},
};

use crate::tid::{TIDCount, TransactionID};

pub struct AprioriHybrid<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl<'a> AprioriHybrid<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
    pub fn run<T: Write>(self, out: &mut T) {
        let mut data = HybridTIDs::<TrieSet>::new(self.data);
        let mut prev = TrieSet::new();
        for n in 1.. {
            match n {
                0 => unreachable!(),
                1 => {
                    let p1: Vec<_> = AprioriP1::new(self.data, self.sup).run_one();
                    for i in 0..p1.len() {
                        if !p1[i] {
                            continue;
                        }
                        out.write_set(&[i]);
                    }
                    continue;
                }
                2 => {
                    prev = AprioriP2::new(self.data, self.sup).run_two();
                }
                3.. => {
                    let mut counter: TrieCounter = prev.join_new();
                    data.count(n, &mut counter);
                    prev = counter.to_frequent_new(self.sup);
                }
            }
            if prev.is_empty() {
                break;
            }
            prev.for_each(|v| {
                out.write_set(v);
            });
        }
    }
}

pub enum HybridIDType<T: TransactionID> {
    ID(T),
    Normal(Vec<usize>),
}

pub struct HybridTID<T: TransactionID> {
    id: HybridIDType<T>,
    change: bool,
}

impl<T: TransactionID + Default> HybridTID<T> {
    pub fn new(id: HybridIDType<T>) -> Self {
        Self { id, change: false }
    }
    pub fn count<U: AprioriCounterMut + AprioriCounting>(&mut self, n: usize, counter: &mut U) {
        match &mut self.id {
            HybridIDType::ID(ids) => {
                if ids.is_empty() {
                    return;
                }
                let new: T = ids.count_new(counter);
                self.id = HybridIDType::ID(new);
            }
            HybridIDType::Normal(items) => {
                if self.change {
                    let mut new = T::default();
                    counter.count_fn(items, n, |v| {
                        new.insert(v);
                    });
                    self.id = HybridIDType::ID(new);
                } else {
                    let mut prev = 0;
                    counter.count_fn(items, n, |_| {
                        prev += 1;
                    });
                    if prev < 100 {
                        self.change = true
                    }
                }
            }
        }
    }
}

pub struct HybridTIDs<T: TransactionID + Default> {
    ids: Vec<HybridTID<T>>,
}

impl<T: TransactionID + Default> HybridTIDs<T> {
    pub fn new(data: &TransactionSet) -> Self {
        Self {
            ids: data
                .iter()
                .cloned()
                .map(|v| HybridTID::new(HybridIDType::Normal(v)))
                .collect(),
        }
    }
    pub fn count<U: AprioriCounterMut + AprioriCounting>(&mut self, n: usize, counter: &mut U) {
        for id in self.ids.iter_mut() {
            id.count(n, counter);
        }
    }
}
