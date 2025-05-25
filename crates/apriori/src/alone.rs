use ahash::AHashMap;

use crate::{start::Write, transaction_set::TransactionSet};

pub struct AprioriTrie {
    data: TransactionSet,
    sup: u64,
}
impl AprioriTrie {
    pub fn new(data: TransactionSet, sup: u64) -> Self {
        Self { data, sup }
    }
    pub fn run(mut self, out: &mut impl Write) {
        let mut prev = Trie::new();
        for k in 1.. {
            if k == 1 {
                for i in 0..self.data.num_items {
                    prev.add(&[i]);
                }
            } else {
                let mut next = Trie::new();
                prev.join(
                    |v| {
                        if v.len() > 2 {
                            let mut pruner: Vec<_> = v.iter().cloned().skip(1).collect();
                            if !prev.contains(&pruner) {
                                return;
                            }
                            for i in 0..(pruner.len() - 2) {
                                pruner[i] = v[i];
                                if !prev.contains(&pruner) {
                                    return;
                                }
                            }
                        }
                        next.add(v);
                    },
                    k - 1,
                );
                prev = next;
            }
            for d in self.data.iter_mut() {
                prev.transaction_count_fn(d, |_| {}, k);
            }
            prev.filter(k, self.sup);
            let mut count = 0;
            prev.for_each(
                |v| {
                    count += 1;
                    out.write_set(v);
                },
                k,
            );
            if count == 0 {
                break;
            }
        }
    }
}

#[derive(Debug)]
pub struct Trie {
    root: Node,
    len: usize,
}

impl Trie {
    pub fn new() -> Self {
        Self {
            root: Node::new(),
            len: 0,
        }
    }
    pub fn add(&mut self, v: &[usize]) -> bool {
        self.len += 1;
        self.root.add(v)
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn join(&self, mut f: impl FnMut(&[usize]), k: usize) {
        let mut v = Vec::new();
        self.root.join(&mut v, &mut f, k);
    }

    pub fn filter(&mut self, n: usize, sup: u64) {
        self.root.filter(n, sup);
    }

    pub fn transaction_count_fn(
        &mut self,
        data: &mut Vec<usize>,
        mut f: impl FnMut(&[usize]),
        k: usize,
    ) {
        let mut v = Vec::new();
        let mut map = AHashMap::new();
        self.root.transaction_count(
            &mut |v| {
                f(v);
                for &n in v {
                    map.entry(n)
                        .and_modify(|c: &mut usize| {
                            *c = c.saturating_add(1);
                        })
                        .or_insert(1usize);
                }
            },
            data,
            &mut v,
            k,
        );
        data.retain(|value| map.get(value).cloned().unwrap_or(0) >= k);
    }

    pub fn get(&self, v: &[usize]) -> Option<u64> {
        self.root.get(v)
    }

    pub fn contains(&self, v: &[usize]) -> bool {
        self.get(v).is_some()
    }

    pub fn for_each(&self, mut f: impl FnMut(&[usize]), k: usize) {
        let mut v = Vec::new();
        self.root.for_each(&mut f, &mut v, k);
    }
}

impl Default for Trie {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug)]
struct Node {
    children: AHashMap<usize, Box<Node>>,
    count: u64,
}
impl Node {
    fn new() -> Self {
        Self {
            children: AHashMap::new(),
            count: 0,
        }
    }

    fn for_each(&self, f: &mut impl FnMut(&[usize]), v: &mut Vec<usize>, k: usize) {
        if k == 0 {
            f(v);
            return;
        }
        for (&a, child) in self.children.iter() {
            v.push(a);
            child.for_each(f, v, k - 1);
            v.pop();
        }
    }

    fn join(&self, v: &mut Vec<usize>, f: &mut impl FnMut(&[usize]), k: usize) {
        if v.len() == k - 1 {
            let a: Vec<_> = self.children.iter().map(|(k, _)| *k).collect();
            for (i, &n1) in a.iter().enumerate() {
                for &n2 in a.iter().skip(i + 1) {
                    let max = n1.max(n2);
                    let min = n1.min(n2);
                    v.push(min);
                    v.push(max);
                    f(v);
                    v.pop();
                    v.pop();
                }
            }
            return;
        }
        for (&a, c) in self.children.iter() {
            v.push(a);
            c.join(v, f, k);
            v.pop();
        }
    }

    fn get(&self, v: &[usize]) -> Option<u64> {
        if v.is_empty() {
            return Some(self.count);
        }
        if let Some(c) = self.children.get(&v[0]) {
            return c.get(&v[1..]);
        }
        None
    }

    fn add(&mut self, v: &[usize]) -> bool {
        if v.is_empty() {
            return false;
        }
        let mut added = false;
        let child = match self.children.get_mut(&v[0]) {
            Some(a) => a,
            None => {
                added = true;
                self.children.insert(v[0], Box::new(Node::new()));
                self.children.get_mut(&v[0]).unwrap()
            }
        };
        let b = child.add(&v[1..]);
        added || b
    }
    fn transaction_count<T>(&mut self, f: &mut T, data: &[usize], v: &mut Vec<usize>, k: usize)
    where
        T: FnMut(&[usize]),
    {
        if v.len() == k {
            self.count += 1;
            f(v);
            return;
        }
        for i in 0..data.len() {
            let child = match self.children.get_mut(&data[i]) {
                Some(a) => a,
                None => continue,
            };
            v.push(data[i]);
            child.transaction_count(f, data, v, k);
            v.pop();
        }
    }

    fn filter(&mut self, n: usize, sup: u64) {
        if n == 1 {
            self.children.retain(|_, v| v.count >= sup);
            return;
        }
        for (_, v) in self.children.iter_mut() {
            v.filter(n - 1, sup);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Trie;

    #[test]
    fn test_add() {
        let mut trie = Trie::new();
        assert!(trie.add(&[1, 2]));
        assert!(!trie.add(&[1]));
        assert!(!trie.add(&[1, 2]));
        assert_eq!(trie.get(&[1, 2]), Some(0));
    }
    #[test]
    fn test_transaction_count() {
        let mut trie = Trie::new();
        trie.add(&[1, 2, 3]);
        trie.add(&[1, 2, 5]);
        trie.add(&[1, 2, 6]);
        let mut transaction = vec![1, 2, 3, 4, 5];
        trie.transaction_count_fn(&mut transaction, |_| {}, 3);
        assert_eq!(trie.get(&[1, 2, 3]), Some(1));
        assert_eq!(trie.get(&[1, 2, 4]), None);
        assert_eq!(trie.get(&[1, 2, 5]), Some(1));
        trie.filter(3, 1);
        assert_eq!(trie.get(&[1, 2, 3]), Some(1));
        assert_eq!(trie.get(&[1, 2, 5]), Some(1));
        assert_eq!(trie.get(&[1, 2, 6]), None);
    }
    #[test]
    fn test_join() {
        let mut trie = Trie::new();
        trie.add(&[1, 2, 3]);
        trie.add(&[1, 2, 5]);
        trie.join(
            |v| {
                assert_eq!(v, &[1, 2, 3, 5]);
            },
            4,
        );
        let mut set = HashSet::new();
        set.insert(vec![1, 2, 3]);
        set.insert(vec![1, 2, 5]);
        trie.for_each(|v| assert!(set.remove(v)), 3);
        assert!(set.is_empty());

        let mut trie = Trie::new();
        trie.add(&[1]);
        trie.add(&[2]);
        trie.join(
            |v| {
                assert_eq!(v, [1, 2]);
            },
            2,
        );
    }
}
