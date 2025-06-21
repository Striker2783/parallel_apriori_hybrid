use std::time::Instant;

use apriori::{
    apriori::AprioriPass1And2,
    start::Write,
    storage::{AprioriCounter, AprioriCounting, AprioriFrequent, Joinable},
    transaction_set::TransactionSet,
    trie::{AprioriTransition, TrieCounter, TrieSet},
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
        let p2 = AprioriPass1And2::new(self.sup, self.data).run(writer);
        let mut prev = AprioriHybridContainer::new(p2, self.sup);
        for n in 3.. {
            let prev_time = Instant::now();
            prev.run(self.data, n);
            println!("{n} {:?}", prev_time.elapsed());
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
pub struct AprioriHybridContainer {
    container: HybridCandidates,
    sup: u64,
    prev: usize,
}
impl AprioriHybridContainer {
    pub fn new(set: TrieSet, sup: u64) -> Self {
        Self {
            container: HybridCandidates::Apriori(set),
            sup,
            prev: 0,
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
                let prev = self.prev;
                let mut trie: TrieCounter = trie_set.join_new();
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
                        if c >= self.sup {
                            candidates.candidates_mut()[i].set_count(c);
                        }
                    });
                    candidates.update_tree(self.sup);
                    self.container = HybridCandidates::Tid(candidates, transformed);
                    return;
                }
                for d in data.iter() {
                    trie.count_fn(d, n, |_| {
                        total += 1;
                    });
                }
                *trie_set = trie.to_frequent_new(self.sup);
            }
            HybridCandidates::Tid(candidates, transformed) => {
                candidates.join_fn(|_| {});
                *transformed = transformed.count(candidates);
                candidates.update_tree(self.sup);
            }
        }
    }
}
