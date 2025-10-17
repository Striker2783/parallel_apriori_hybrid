use std::{
    collections::HashSet,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use apriori::transaction_set::TransactionSet;
pub const SOLVED: &str = "solve1.dat";
pub const DATABASE: &str = "test1.dat";
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
impl From<HashSet<Vec<usize>>> for Solved {
    fn from(other: HashSet<Vec<usize>>) -> Self {
        Self { set: other }
    }
}

pub fn test_generic<T: AsRef<Path>>(test_files: T, f: impl Fn(TransactionSet, u64) -> Solved) {
    test_generic_with_option(test_files, |t, s| Some(f(t, s)));
}

pub fn test_generic_with_option<T: AsRef<Path>>(
    test_files: T,
    f: impl Fn(TransactionSet, u64) -> Option<Solved>,
) {
    test_generic_with_paths(test_files, |path, sup| {
        let data = TransactionSet::from_path(&path).unwrap();
        f(data, sup)
    });
}

pub fn test_generic_with_paths<T: AsRef<Path>>(
    test_files: T,
    f: impl Fn(PathBuf, u64) -> Option<Solved>,
) {
    let database = test_files.as_ref().join(DATABASE);
    let solved = test_files.as_ref().join(SOLVED);
    assert!(database.exists(), "Database file does not exist");

    let s = f(database.clone(), 10);
    assert!(solved.exists(), "Solved file does not exist");
    if s.is_none() {
        return;
    }
    let s = s.unwrap();
    let f = File::open(solved).expect("Solved File Not Found");
    let s2 = Solved::from_file(f).expect("Invalid Format Error");
    assert_eq!(s.set.len(), s2.set.len());
    assert_eq!(s, s2);
}
