use std::{fs::File, io::{BufRead, BufReader}, ops::{Deref, DerefMut}};

/// A 0-indexed item set
/// A Transactional Database
#[derive(Debug, Default)]
pub struct TransactionSet {
    pub transactions: Vec<Vec<usize>>,
    pub num_items: usize
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
        Self { transactions, num_items }
    }
    /// Iterates over all the transactions
    pub fn iter(&self) -> impl Iterator<Item = &Vec<usize>> {
        self.transactions.iter()
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
            let mut items: Vec<usize> = line.split_whitespace().map(|s| s.parse::<usize>().unwrap()).collect();
            // Sorts the items and sets the max
            items.sort();
            max = (*items.iter().max().unwrap()).max(max);
            transactions.push(items);
        }
        Self { transactions, num_items: max + 1 }
    }
}