use std::time::Instant;

use crate::array2d::AprioriP2Counter;
use crate::start::Apriori;
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

pub struct AprioriRunner<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl Apriori for AprioriRunner<'_> {
    fn run<T: Write>(self, out: &mut T) {
        let p1 = apriori_pass_one(self.data, self.sup);
        p1.iter().for_each(|&n| out.write_set(&[n]));
        let mut prev: TrieSet = apriori_pass_two(self.data, self.sup, &p1);
        prev.for_each(|v| out.write_set(v));
        for i in 3.. {
            let prev_time = Instant::now();
            prev = apriori_pass_three(self.data, &prev, i, self.sup);
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
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}

pub fn apriori_pass_two_counter(data: &TransactionSet, counter: &mut impl AprioriCounter) {
    for d in data.iter() {
        for (i, a) in d.iter().cloned().enumerate() {
            for b in d.iter().cloned().skip(i + 1) {
                counter.increment(&[a, b]);
            }
        }
    }
}
pub fn apriori_pass_two<T: AprioriFrequent + Default>(
    data: &TransactionSet,
    sup: u64,
    freq: &[usize],
) -> T {
    let mut counter = AprioriP2Counter::new(freq);
    apriori_pass_two_counter(data, &mut counter);
    counter.to_frequent_new::<T>(sup)
}
pub fn apriori_pass_three_counter<T: AprioriCounting>(data: &TransactionSet, counter: &mut T, n: usize) {
    for d in data.iter() {
        counter.count(d, n);
    }
}
pub fn apriori_pass_three<T: AprioriFrequent + Default>(
    data: &TransactionSet,
    prev: &impl AprioriFrequent,
    n: usize,
    sup: u64,
) -> T {
    let mut counter: TrieCounter = prev.join_new();
    apriori_pass_three_counter(data, &mut counter, n);
    counter.to_frequent_new(sup)
}
#[cfg(test)]
mod tests {

    use crate::{
        apriori::{apriori_pass_one, apriori_pass_three, apriori_pass_two},
        start::{Apriori, FrequentWriter},
        storage::AprioriFrequent,
        transaction_set::TransactionSet,
        trie::TrieSet,
    };

    use super::AprioriRunner;

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
        let a: TrieSet = apriori_pass_two(&set, 2, &apriori_pass_one(&set, 2));
        assert_eq!(a.len(), 1);
        assert!(a.contains(&[2, 3]));
        let a: TrieSet = apriori_pass_two(&set, 2, &[1, 2, 3]);
        assert_eq!(a.len(), 1);
        assert!(a.contains(&[2, 3]));
    }
    #[test]
    fn test_run_general() {
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![1, 2, 3]], 4);
        let mut frequent = TrieSet::new();
        frequent.insert(&[1, 2]);
        frequent.insert(&[1, 3]);
        frequent.insert(&[2, 3]);
        let f: TrieSet = apriori_pass_three(&set, &frequent, 3, 2);
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
