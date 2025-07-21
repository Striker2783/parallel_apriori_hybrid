use std::time::Instant;

use crate::array2d::{AprioriP2Counter, AprioriP2Counter2};
use crate::start::{Apriori, AprioriGeneral, AprioriTwo};
use crate::storage::{AprioriCounter, AprioriCounting, AprioriFrequent};
use crate::trie::{TrieCounter, TrieSet};
use crate::{start::Write, transaction_set::TransactionSet};

pub fn apriori_pass_one_counter(data: &TransactionSet, counter: &mut impl AprioriCounter) {
    for d in data.iter() {
        for &n in d.iter() {
            counter.increment(&[n]);
        }
    }
}
pub fn apriori_pass_one(data: &TransactionSet, sup: u64) -> Vec<usize> {
    let mut counter = vec![0; data.num_items];
    apriori_pass_one_counter(data, &mut counter);
    counter
        .iter()
        .enumerate()
        .filter(|(_, count)| **count >= sup)
        .map(|(i, _)| i)
        .collect()
}
pub struct AprioriPass1And2<'a> {
    sup: u64,
    data: &'a TransactionSet,
}
impl<'a> AprioriPass1And2<'a> {
    pub fn new(sup: u64, data: &'a TransactionSet) -> Self {
        Self { sup, data }
    }
    fn pass_1(&self, out: &mut impl Write) -> Vec<usize> {
        let p1: Vec<_> = apriori_pass_one(self.data, self.sup);
        p1.iter().for_each(|&n| {
            out.write_set(&[n]);
        });
        p1
    }
    pub fn count(self, out: &mut impl Write) -> TrieCounter {
        let p1 = self.pass_1(out);
        let p2 = AprioriP2New::new(self.data, &p1, self.sup).count();
        p2.for_each(|v, c| {
            if c < self.sup {
                return;
            }
            out.write_set(v);
        });
        p2
    }
    pub fn run(self, out: &mut impl Write) -> TrieSet {
        let p1 = self.pass_1(out);
        let p2 = AprioriP2New::new(self.data, &p1, self.sup).run();
        p2.for_each(|v| {
            out.write_set(v);
        });
        p2
    }
}

pub struct AprioriRunner<'a> {
    data: &'a mut TransactionSet,
    sup: u64,
}

impl Apriori for AprioriRunner<'_> {
    fn run<T: Write>(self, out: &mut T) {
        let mut prev = AprioriPass1And2::new(self.sup, self.data).run(out);
        for i in 3.. {
            let prev_time = Instant::now();
            prev = AprioriP3::new(self.data, self.sup).run(&prev, i);
            println!("{i} {:?}", prev_time.elapsed());
            if prev.is_empty() {
                break;
            }
            prev.for_each(|v| {
                out.write_set(v);
            });
        }
    }
}

impl<'a> AprioriRunner<'a> {
    pub fn new(data: &'a mut TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}

pub struct AprioriP2New<'a> {
    data: &'a TransactionSet,
    frequent: &'a [usize],
    sup: u64,
}

impl<'a> AprioriP2New<'a> {
    pub fn new(data: &'a TransactionSet, frequent: &'a [usize], sup: u64) -> Self {
        Self {
            data,
            frequent,
            sup,
        }
    }
    pub fn run(self) -> TrieSet {
        let mut counter = AprioriP2Counter2::new(self.frequent);
        for data in self.data.iter() {
            for (i, a) in data.iter().cloned().enumerate() {
                for b in data.iter().cloned().skip(i + 1) {
                    counter.increment(&[a, b]);
                }
            }
        }
        counter.to_frequent_new(self.sup)
    }
    pub fn count(self) -> TrieCounter {
        let mut counter = AprioriP2Counter2::new(self.frequent);
        for data in self.data.iter() {
            for (i, a) in data.iter().cloned().enumerate() {
                for b in data.iter().cloned().skip(i + 1) {
                    counter.increment(&[a, b]);
                }
            }
        }
        let mut new_counter = TrieCounter::new();
        counter.for_each(|v, c| {
            if c < self.sup {
                return;
            }
            new_counter.add(v, c);
        });
        new_counter
    }
}
pub struct AprioriP2<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl AprioriTwo<TrieSet> for AprioriP2<'_> {
    fn run_two(self) -> TrieSet {
        let mut counter = AprioriP2Counter::new(self.data.num_items);
        for data in self.data.iter() {
            for (i, a) in data.iter().cloned().enumerate() {
                for b in data.iter().cloned().skip(i + 1) {
                    counter.increment(&[a, b]);
                }
            }
        }
        counter.to_frequent_new(self.sup)
    }
}

impl<'a> AprioriP2<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}

pub struct AprioriP3<'a> {
    data: &'a mut TransactionSet,
    sup: u64,
}

impl<'a> AprioriP3<'a> {
    pub fn new(data: &'a mut TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}

impl AprioriGeneral<TrieSet> for AprioriP3<'_> {
    fn run(self, trie: &impl AprioriFrequent, n: usize) -> TrieSet {
        let mut trie: TrieCounter = trie.join_new();
        for d in self.data.iter() {
            trie.count(d, n);
        }
        trie.to_frequent_new(self.sup)
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        apriori::{AprioriP2New, apriori_pass_one},
        start::{Apriori, AprioriGeneral, AprioriTwo, FrequentWriter},
        storage::AprioriFrequent,
        transaction_set::TransactionSet,
        trie::TrieSet,
    };

    use super::{AprioriP2, AprioriP3, AprioriRunner};

    #[test]
    fn test_run_one() {
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![2, 3]], 4);
        let a: Vec<_> = apriori_pass_one(&set, 2);
        assert!(!a.contains(&0));
        assert!(!a.contains(&1));
        assert!(a.contains(&2));
        assert!(a.contains(&3));
    }
    #[test]
    fn test_run_two() {
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![2, 3]], 4);
        let a = AprioriP2::new(&set, 2).run_two();
        assert_eq!(a.len(), 1);
        assert!(a.contains(&[2, 3]));
        let a = AprioriP2New::new(&set, &[1, 2, 3], 2).run();
        assert_eq!(a.len(), 1);
        assert!(a.contains(&[2, 3]));
    }
    #[test]
    fn test_run_general() {
        let mut set = TransactionSet::new(vec![vec![1, 2, 3], vec![1, 2, 3]], 4);
        let a = AprioriP3::new(&mut set, 2);
        let mut frequent = TrieSet::new();
        frequent.insert(&[1, 2]);
        frequent.insert(&[1, 3]);
        frequent.insert(&[2, 3]);
        let f = a.run(&frequent, 3);
        assert_eq!(f.len(), 1);
        assert!(f.contains(&[1, 2, 3]));
    }
    #[test]
    fn test_run_apriori() {
        let mut set = TransactionSet::new(vec![vec![1, 2, 3], vec![1, 2, 3]], 4);
        let a = AprioriRunner::new(&mut set, 2);
        let mut s = FrequentWriter::<TrieSet>::new();
        a.run(&mut s);
        let s = s.into_inner();
        assert!(s.contains(&[1, 2, 3]));
        assert!(s.contains(&[1, 2]));
        assert!(s.contains(&[1, 3]));
        assert!(s.contains(&[2, 3]));
        assert!(s.contains(&[1]));
        assert!(s.contains(&[2]));
        assert!(s.contains(&[3]));
        assert!(!s.contains(&[0]));
        assert_eq!(s.len(), 7);
    }
}
