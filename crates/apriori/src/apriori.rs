use crate::array2d::AprioriP2Counter;
use crate::count::{AprioriCounting, Count};
use crate::start::{Apriori, AprioriGeneral, AprioriTwo};
use crate::storage::{AprioriCounter, AprioriFrequent};
use crate::trie::{TrieCounter, TrieSet};
use crate::{
    start::{AprioriOne, Write},
    transaction_set::TransactionSet,
};

pub struct AprioriRunner<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl Apriori for AprioriRunner<'_> {
    fn run<T: Write>(self, out: &mut T) {
        let mut prev = TrieSet::new();
        for i in 1.. {
            match i {
                0 => unreachable!(),
                1 => {
                    let p1 = AprioriP1::new(self.data, self.sup).run_one();
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
                    prev = AprioriP3::new(self.data, self.sup).run(&prev, i);
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

impl<'a> AprioriRunner<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}

pub struct AprioriP1<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl<'a> AprioriP1<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}
impl AprioriOne<Vec<bool>> for AprioriP1<'_> {
    fn run_one(self) -> Vec<bool> {
        let mut counter = vec![0u64; self.data.num_items];
        for data in self.data.iter() {
            for &i in data {
                counter.increment(&[i]);
            }
        }
        let mut frequent = vec![false; self.data.num_items];
        counter.to_frequent(&mut frequent, self.sup);
        frequent
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
    data: &'a TransactionSet,
    sup: u64,
}

impl<'a> AprioriP3<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}

impl AprioriGeneral<TrieSet> for AprioriP3<'_> {
    fn run(self, trie: &impl AprioriFrequent, n: usize) -> TrieSet {
        let mut trie: TrieCounter = trie.join_new();
        let counter = AprioriCounting::new(self.data, &mut trie);
        counter.count(n);
        trie.to_frequent_new(self.sup)
    }
}
#[cfg(test)]
mod tests {

    use crate::{
        start::{Apriori, AprioriGeneral, AprioriOne, AprioriTwo, FrequentWriter},
        storage::AprioriFrequent,
        transaction_set::TransactionSet,
        trie::TrieSet,
    };

    use super::{AprioriP1, AprioriP2, AprioriP3, AprioriRunner};

    #[test]
    fn test_run_one() {
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![2, 3]], 4);
        let a = AprioriP1::new(&set, 2).run_one();
        assert_eq!(a.len(), 2);
        assert!(!a.contains(&[0]));
        assert!(!a.contains(&[1]));
        assert!(a.contains(&[2]));
        assert!(a.contains(&[3]));
    }
    #[test]
    fn test_run_two() {
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![2, 3]], 4);
        let a = AprioriP2::new(&set, 2).run_two();
        assert_eq!(a.len(), 1);
        assert!(a.contains(&[2, 3]));
    }
    #[test]
    fn test_run_general() {
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![1, 2, 3]], 4);
        let a = AprioriP3::new(&set, 2);
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
        let set = TransactionSet::new(vec![vec![1, 2, 3], vec![1, 2, 3]], 4);
        let a = AprioriRunner::new(&set, 2);
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
