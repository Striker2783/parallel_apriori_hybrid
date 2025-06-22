use std::time::Instant;

use apriori::{
    apriori::AprioriPass1And2,
    start::Write,
    storage::{AprioriCounter, AprioriCounting, AprioriFrequent, Joinable},
    transaction_set::TransactionSet,
    trie::{AprioriTransition, TrieCounter, TrieSet},
};
use parallel::traits::Convertable;

use crate::tid::{CandidateID, Candidates, TransformedDatabase};

pub struct AprioriHybridRunner<'a> {
    data: &'a mut TransactionSet,
    sup: u64,
}

impl<'a> AprioriHybridRunner<'a> {
    pub fn new(data: &'a mut TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
    pub fn run<T: Write>(self, writer: &mut T) {
        let p2 = AprioriPass1And2::new(self.sup, self.data).count(writer);
        let mut prev = AprioriHybridContainer::new(p2, self.sup);
        for n in 3.. {
            let prev_time = Instant::now();
            prev.run(self.data, n);
            println!("{n} {:?}", prev_time.elapsed());
            let mut total = 0;
            prev.for_each(|v, c| {
                if c < self.sup {
                    return;
                }
                total += 1;
                writer.write_set(v);
            });
            if total == 0 {
                break;
            }
        }
    }
}
enum HybridCandidates {
    Apriori(TrieCounter),
    Tid(Candidates, TransformedDatabase),
}
pub struct AprioriHybridContainer {
    container: HybridCandidates,
    sup: u64,
    prev: usize,
}
impl AprioriHybridContainer {
    pub fn new(set: TrieCounter, sup: u64) -> Self {
        Self {
            container: HybridCandidates::Apriori(set),
            sup,
            prev: 0,
        }
    }
    pub fn for_each(&self, mut f: impl FnMut(&[usize], u64)) {
        match &self.container {
            HybridCandidates::Apriori(trie_set) => trie_set.for_each(f),
            HybridCandidates::Tid(candidates, _) => {
                candidates.for_each_range(|c| f(c.items(), c.count()))
            }
        }
    }
    pub fn set(&mut self, set: &TrieSet) {
        match &mut self.container {
            HybridCandidates::Apriori(trie_counter) => {
                set.for_each(|v| {
                    trie_counter.add(v, self.sup);
                });
            }
            HybridCandidates::Tid(candidates, _) => {
                candidates.for_each_range_mut(|candidate| {
                    if set.contains(candidate.items()) {
                        candidate.set_count(self.sup);
                    }
                });
            }
        }
    }
    pub fn run(&mut self, data: &mut TransactionSet, n: usize) {
        match &mut self.container {
            HybridCandidates::Apriori(trie_set) => {
                let prev = self.prev;
                let trie: TrieSet = trie_set.to_frequent_new(self.sup);
                let mut trie: TrieCounter = trie.join_new();
                let mut total = 0;
                self.prev = trie.len();
                if self.prev < prev && prev < 100_000 {
                    println!("SWITCH");
                    let mut transition = AprioriTransition::new();
                    let mut candidates = Candidates::new(self.sup);
                    trie.for_each(|v, _| {
                        let index = candidates.candidates().len();
                        let candidate = CandidateID::new(v.to_vec(), (usize::MAX, usize::MAX));
                        candidates.push(candidate);
                        transition.insert(v, index);
                    });
                    let transformed = TransformedDatabase::transition(data, &mut transition, n);
                    transition.for_each(|_, (i, c)| {
                        candidates.candidates_mut()[i].set_count(c);
                    });
                    self.container = HybridCandidates::Tid(candidates, transformed);
                    return;
                }
                for d in data.iter() {
                    trie.count_fn(d, n, |_| {
                        total += 1;
                    });
                }
                *trie_set = trie;
            }
            HybridCandidates::Tid(candidates, transformed) => {
                candidates.update_tree(self.sup);
                candidates.join_fn(|_| {});
                *transformed = transformed.count(candidates);
            }
        }
    }
}
impl Convertable for AprioriHybridContainer {
    fn to_vec(&mut self) -> Vec<u64> {
        match &mut self.container {
            HybridCandidates::Apriori(trie_counter) => trie_counter.to_vec(),
            HybridCandidates::Tid(candidates, _) => candidates.to_vec(),
        }
    }

    fn add_from_vec(&mut self, v: &[u64]) {
        match &mut self.container {
            HybridCandidates::Apriori(trie_counter) => trie_counter.add_from_vec(v),
            HybridCandidates::Tid(candidates, _) => candidates.add_from_vec(v),
        }
    }
}
