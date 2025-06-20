use apriori::{
    apriori::{AprioriP1, AprioriP2New, AprioriP3},
    start::{AprioriGeneral, AprioriOne, Write},
    storage::{AprioriFrequent, Joinable},
    transaction_set::TransactionSet,
    trie::{AprioriTransition, TrieSet},
};

use crate::tid2::{CandidateID, Candidates, TransformedDatabase};

pub struct AprioriHybridRunner<'a> {
    data: &'a mut TransactionSet,
    sup: u64,
}

impl<'a> AprioriHybridRunner<'a> {
    pub fn new(data: &'a mut TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
    pub fn run<T: Write>(self, writer: &mut T) {
        let p1: Vec<_> = AprioriP1::new(self.data, self.sup).run_one();
        for i in 0..p1.len() {
            if !p1[i] {
                continue;
            }
            writer.write_set(&[i]);
        }
        let p1: Vec<_> = p1
            .iter()
            .enumerate()
            .filter(|(_, count)| **count)
            .map(|(i, _)| i)
            .collect();
        let p2 = AprioriP2New::new(self.data, &p1, self.sup).run();
        p2.for_each(|v| {
            writer.write_set(v);
        });
        let mut prev = AprioriHybridContainer::new(p2, self.sup);
        for n in 3.. {
            prev.run(self.data, n);
            let mut total = 0;
            prev.for_each(|v| {
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
    Apriori(TrieSet),
    Tid(Candidates, TransformedDatabase),
}
const SWITCH: usize = 5;
pub struct AprioriHybridContainer {
    container: HybridCandidates,
    sup: u64,
}
impl AprioriHybridContainer {
    pub fn new(set: TrieSet, sup: u64) -> Self {
        Self {
            container: HybridCandidates::Apriori(set),
            sup,
        }
    }
    pub fn for_each(&self, mut f: impl FnMut(&[usize])) {
        match &self.container {
            HybridCandidates::Apriori(trie_set) => trie_set.for_each(f),
            HybridCandidates::Tid(candidates, _) => candidates.for_each_range(|c| {
                if c.count() >= self.sup {
                    f(c.items())
                }
            }),
        }
    }
    pub fn run(&mut self, data: &mut TransactionSet, n: usize) {
        match &mut self.container {
            HybridCandidates::Apriori(trie_set) => {
                if n == SWITCH {
                    let trie_set = AprioriP3::new(data, self.sup).run(trie_set, n);
                    let mut transition = AprioriTransition::new();
                    let mut candidates = Candidates::new(self.sup);
                    trie_set.for_each(|v| {
                        let index = candidates.candidates().len();
                        let mut candidate = CandidateID::new(v.to_vec(), (usize::MAX, usize::MAX));
                        candidate.set_count(self.sup);
                        candidates.push(candidate);
                        transition.insert(v, index);
                    });
                    candidates.update_tree(self.sup);
                    let transformed = TransformedDatabase::transition(data, &mut transition, n);
                    self.container = HybridCandidates::Tid(candidates, transformed);
                } else {
                    *trie_set = AprioriP3::new(data, self.sup).run(trie_set, n);
                }
            }
            HybridCandidates::Tid(candidates, transformed) => {
                candidates.join_fn(|_| {});
                *transformed = transformed.count(candidates);
                candidates.update_tree(self.sup);
            }
        }
    }
}
