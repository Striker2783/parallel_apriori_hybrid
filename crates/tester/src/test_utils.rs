use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use apriori::transaction_set::TransactionSet;
pub const SOLVED: &str = "../../test_files/solve1.dat";
pub const DATABASE: &str = "../../test_files/test1.dat";
#[derive(Debug)]
pub enum FromFileError {
    InvalidFormat(std::num::ParseIntError),
    IO(std::io::Error),
}
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Solved {
    pub set: HashSet<Vec<usize>>,
}
impl Solved {
    pub fn new(set: HashSet<Vec<usize>>) -> Self {
        Self { set }
    }
    pub fn from_file(file: File) -> Result<Self, FromFileError> {
        let mut this = Self::default();
        for l in BufReader::new(file).lines() {
            if l.is_err() {
                continue;
            }
            let l = l.unwrap();
            let t: Vec<_> = l
                .split_whitespace()
                .filter_map(|n| n.parse::<usize>().ok())
                .collect();
            this.set.insert(t);
        }
        Ok(this)
    }
}

pub fn test_generic(f: impl Fn(TransactionSet, u64) -> Solved) {
    let path = Path::new(DATABASE);
    if !path.exists() {
        panic!("Database file does not exist");
    }
    let data = File::open(path).unwrap();
    let t = TransactionSet::from_dat(data);

    let s = f(t, 10);
    let path = Path::new(SOLVED);
    if !path.exists() {
        panic!("Solved file does not exist");
    }
    let f = File::open(SOLVED).expect("Solved File Not Found");
    let s2 = Solved::from_file(f).expect("Invalid Format Error");
    assert_eq!(s.set.len(), s2.set.len());
    assert_eq!(s, s2);
}
