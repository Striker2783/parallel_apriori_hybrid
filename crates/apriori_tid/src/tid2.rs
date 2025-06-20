use std::{
    collections::{HashMap, HashSet},
    ops::Range,
    time::Instant,
};

use apriori::{
    start::Write,
    storage::{AprioriFrequent, Joinable},
    transaction_set::TransactionSet,
    trie::{AprioriTransition, TrieSet},
};

pub struct AprioriTIDRunner2<'a> {
    data: &'a TransactionSet,
    sup: u64,
}

impl<'a> AprioriTIDRunner2<'a> {
    pub fn new(data: &'a TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
    pub fn run<T: Write>(self, writer: &mut T) {
        let mut c = Candidates::new(self.sup);
        for i in 0..self.data.num_items {
            c.push(CandidateID::new(vec![i], (usize::MAX, usize::MAX)));
        }
        for d in self.data.iter() {
            for &n in d {
                c.candidates_mut()[n].count += 1;
            }
        }
        c.for_each_range(|a| {
            if a.count >= self.sup {
                writer.write_set(&a.items);
            }
        });
        c.update_tree(self.sup);
        c.join_fn(|_| {});
        let mut transformed: TransformedDatabase = self.data.into();
        for _n in 2usize.. {
            transformed = transformed.count(&mut c);
            if c.prev.is_empty() {
                break;
            }
            c.for_each_range(|a| {
                if a.count >= self.sup {
                    writer.write_set(&a.items);
                }
            });
            c.update_tree(self.sup);
            c.join_fn(|_| {});
        }
    }
}
#[derive(Debug)]
pub struct TransformedDatabase {
    v: Vec<HashSet<usize>>,
}

impl std::ops::DerefMut for TransformedDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.v
    }
}

impl std::ops::Deref for TransformedDatabase {
    type Target = Vec<HashSet<usize>>;

    fn deref(&self) -> &Self::Target {
        &self.v
    }
}

impl TransformedDatabase {
    pub fn transition(data: &TransactionSet, transition: &mut AprioriTransition, n: usize) -> Self {
        let mut a = Self::new();
        for d in data.iter() {
            let mut set = HashSet::new();
            transition.count_fn(d, n, |i| {
                set.insert(i);
            });
            if set.is_empty() {
                continue;
            }
            a.push(set);
        }
        a
    }
    pub fn count(&self, c: &mut Candidates) -> Self {
        let mut new = Self::new();
        for set in &self.v {
            let mut new_set = HashSet::new();
            for &n in set {
                let data = &c.candidates[n];
                for &ext in &data.extensions {
                    let extended = &c.candidates[ext];
                    if new_set.contains(&ext) {
                        continue;
                    }
                    if set.contains(&extended.generators.0) && set.contains(&extended.generators.1)
                    {
                        new_set.insert(ext);
                    }
                }
            }
            if !new_set.is_empty() {
                for &i in new_set.iter() {
                    c.candidates[i].count += 1;
                }
                new.push(new_set);
            }
        }
        new
    }
}

impl From<&TransactionSet> for TransformedDatabase {
    fn from(value: &TransactionSet) -> Self {
        Self {
            v: value
                .transactions
                .iter()
                .map(|v| {
                    let mut set = HashSet::new();
                    for n in v {
                        set.insert(*n);
                    }
                    set
                })
                .collect(),
        }
    }
}

impl TransformedDatabase {
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }
}

impl Default for TransformedDatabase {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Candidates {
    candidates: Vec<CandidateID>,
    tree: TrieSet,
    prev: Range<usize>,
    sup: u64,
}
impl Candidates {
    pub fn candidates(&self) -> &Vec<CandidateID> {
        &self.candidates
    }
    pub fn new(sup: u64) -> Self {
        Self {
            sup,
            tree: TrieSet::new(),
            candidates: Vec::new(),
            prev: 0..0,
        }
    }
    pub fn update_tree(&mut self, sup: u64) {
        for i in self.prev.clone() {
            let c = &self.candidates[i];
            if c.count >= sup {
                self.tree.insert(&c.items);
            }
        }
    }
    fn prune(&self, v: &[usize]) -> bool {
        let mut pruner: Vec<_> = v.iter().cloned().skip(1).collect();
        if !self.tree.contains(&pruner) {
            return true;
        }
        if pruner.len() < 2 {
            return false;
        }
        for i in 0..(pruner.len() - 2) {
            pruner[i] = v[i];
            if !self.tree.contains(&pruner) {
                return true;
            }
        }
        false
    }
    pub fn push(&mut self, value: CandidateID) {
        let id = self.candidates.len();
        self.prev = self.prev.start..(self.prev.end + 1);
        for g in [value.generators.0, value.generators.1] {
            if g == usize::MAX {
                continue;
            }
            self.candidates[g].extensions.insert(id);
        }
        self.candidates.push(value)
    }
    pub fn for_each_range(&self, mut f: impl FnMut(&CandidateID)) {
        for i in self.prev.clone() {
            f(&self.candidates[i])
        }
    }

    pub fn candidates_mut(&mut self) -> &mut Vec<CandidateID> {
        &mut self.candidates
    }
}

impl Joinable<CandidateID> for Candidates {
    fn join_fn<U: FnMut(CandidateID)>(&mut self, mut f: U) {
        let mut map: HashMap<Vec<usize>, Vec<(usize, usize)>> = HashMap::new();
        for i in self.prev.clone() {
            if self.candidates[i].count < self.sup {
                continue;
            }
            let mut v = self.candidates[i].items.clone();
            let last = v.pop().unwrap();
            match map.get_mut(&v) {
                Some(vec) => vec.push((last, i)),
                None => {
                    map.insert(v, vec![(last, i)]);
                }
            }
        }
        self.prev = self.prev.end..self.prev.end;
        for (mut prefix, vec) in map.into_iter() {
            for (i, c1) in vec.iter().enumerate() {
                for c2 in vec.iter().skip(i + 1) {
                    prefix.push(c1.0.min(c2.0));
                    prefix.push(c1.0.max(c2.0));
                    if !self.prune(&prefix) {
                        let c = CandidateID::new(prefix.clone(), (c1.1, c2.1));
                        f(c.clone());
                        self.push(c);
                    }
                    prefix.pop();
                    prefix.pop();
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CandidateID {
    generators: (usize, usize),
    extensions: HashSet<usize>,
    count: u64,
    items: Vec<usize>,
}

impl CandidateID {
    pub fn new(items: Vec<usize>, generators: (usize, usize)) -> Self {
        Self {
            items,
            generators,
            extensions: HashSet::new(),
            count: 0,
        }
    }

    pub fn generators(&self) -> (usize, usize) {
        self.generators
    }

    pub fn extensions(&self) -> &HashSet<usize> {
        &self.extensions
    }

    pub fn count(&self) -> u64 {
        self.count
    }

    pub fn items(&self) -> &[usize] {
        &self.items
    }
    
    pub fn set_count(&mut self, count: u64) {
        self.count = count;
    }
}
