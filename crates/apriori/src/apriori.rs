use crate::{start::AprioriOne, transaction_set::TransactionSet};
use crate::{storage::AprioriCounter};

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
#[cfg(test)]
mod tests {
    use crate::{start::AprioriOne, storage::AprioriFrequent, transaction_set::TransactionSet};

    use super::AprioriP1;

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
}