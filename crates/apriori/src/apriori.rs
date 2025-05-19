use crate::array2d::AprioriP2Counter;
use crate::start::AprioriTwo;
use crate::storage::{AprioriCounter, AprioriFrequent};
use crate::trie::TrieSet;
use crate::{start::AprioriOne, transaction_set::TransactionSet};

pub struct AprioriP1<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl<'a> AprioriP1<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}
impl AprioriOne for AprioriP1<'_> {
    fn run_one(self) -> impl crate::storage::AprioriFrequent {
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

impl<'a> AprioriTwo for AprioriP2<'a> {
    fn run_two(self) -> impl crate::storage::AprioriFrequent {
        let mut counter = AprioriP2Counter::new(self.data.num_items);
        for data in self.data.iter() {
            for (i, a) in data.iter().cloned().enumerate() {
                for b in data.iter().cloned().skip(i + 1) {
                    counter.increment(&[a, b]);
                }
            }
        }
        counter.to_frequent_new::<TrieSet>(self.sup)
    }
}

impl<'a> AprioriP2<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
}
#[cfg(test)]
mod tests {
    use crate::{
        start::{AprioriOne, AprioriTwo},
        storage::AprioriFrequent,
        transaction_set::TransactionSet,
    };

    use super::{AprioriP1, AprioriP2};

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
}
