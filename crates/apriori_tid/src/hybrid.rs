use apriori::{
    storage::{AprioriCounterMut, AprioriCounting},
    transaction_set::TransactionSet,
};

use crate::tid::{TIDCount, TransactionID};

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
