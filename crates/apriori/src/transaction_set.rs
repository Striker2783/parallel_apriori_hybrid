use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
    ops::{Deref, DerefMut},
    path::Path,
};

/// A 0-indexed item set
/// A Transactional Database
#[derive(Debug, Default)]
pub struct TransactionSet {
    pub transactions: Vec<Vec<usize>>,
    pub num_items: usize,
    pub size: usize,
}
// Dereferences to the underlying Vector
impl Deref for TransactionSet {
    type Target = Vec<Vec<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.transactions
    }
}
impl DerefMut for TransactionSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transactions
    }
}

impl TransactionSet {
    /// Constructor
    pub fn new(transactions: Vec<Vec<usize>>, num_items: usize) -> Self {
        let size = transactions.iter().map(|v| v.len()).sum();
        Self {
            transactions,
            num_items,
            size,
        }
    }
    pub fn partition(&self, ranks: usize) -> TransactionSetPartitioner {
        TransactionSetPartitioner::new(ranks, self)
    }
    /// Iterates over all the transactions
    pub fn iter(&self) -> impl Iterator<Item = &Vec<usize>> {
        self.transactions.iter()
    }
    pub fn from_path(path: &Path) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(Self::from_dat(file))
    }
    /// Constructs the set from a .dat file
    /// .dat file is a file with one transaction per line.
    /// Each transaction is a space-separated list of ids.
    pub fn from_dat(f: File) -> Self {
        let mut max = 0;
        let mut transactions = Vec::new();
        // Loops through each line of the file
        for l in BufReader::new(f).lines() {
            if l.is_err() {
                continue;
            }
            let line = l.unwrap();
            // Parses the transaction
            let mut items: Vec<usize> = line
                .split_whitespace()
                .map(|s| s.parse::<usize>().unwrap())
                .collect();
            // Sorts the items and sets the max
            items.sort();
            items.dedup();
            max = (*items.iter().max().unwrap()).max(max);
            transactions.push(items);
        }
        Self::new(transactions, max + 1)
    }
}

pub struct TransactionSetPartitioner<'a> {
    size: usize,
    original: &'a TransactionSet,
    curr: usize,
}

impl<'a> TransactionSetPartitioner<'a> {
    pub fn new(ranks: usize, original: &'a TransactionSet) -> Self {
        Self {
            size: ranks,
            original,
            curr: 0,
        }
    }
}

impl Iterator for TransactionSetPartitioner<'_> {
    type Item = TransactionSet;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.size {
            return None;
        }
        let size = self.original.len() / self.size;
        let begin = self.curr * size;
        let end = ((self.curr + 1) * size).min(self.original.len());
        let v = &self.original[begin..end];
        let v: Vec<_> = v.to_vec();
        let t = TransactionSet::new(v, self.original.num_items);
        self.curr += 1;
        Some(t)
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction_set::TransactionSet;

    #[test]
    fn test_paritioning() {
        let v = vec![vec![1, 2, 3], vec![2, 3, 4], vec![3, 4, 5]];
        let t = TransactionSet::new(v, 5);
        let mut a = t.partition(3);
        assert_eq!(a.next().unwrap().transactions, vec![vec![1, 2, 3]]);
        assert_eq!(a.next().unwrap().transactions, vec![vec![2, 3, 4]]);
        assert_eq!(a.next().unwrap().transactions, vec![vec![3, 4, 5]]);
        assert!(a.next().is_none());
    }
}
