use std::ops::{Deref, DerefMut};

use fnv::FnvHashMap;
use parallel::traits::Convertable;

use crate::storage::{AprioriCounter, AprioriCounterMut, AprioriCounting, AprioriFrequent};

pub struct TrieSet(Trie<bool>, usize);

impl Convertable for TrieSet {
    fn to_vec(&mut self) -> Vec<u64> {
        let mut v = Vec::new();
        self.0.to_vec(&mut v);
        v
    }

    fn add_from_vec(&mut self, v: &[u64]) {
        self.0.add_from_vec(v);
    }
}

impl Default for TrieSet {
    fn default() -> Self {
        Self(Trie::new(false), 0)
    }
}

impl TrieSet {
    pub fn new() -> Self {
        Self::default()
    }
}
impl AprioriFrequent for TrieSet {
    fn for_each(&self, mut f: impl FnMut(&[usize])) {
        self.0.for_each(|v, b| {
            if b {
                f(v);
            }
        });
    }

    fn contains(&self, v: &[usize]) -> bool {
        self.0.get(v).unwrap_or(false)
    }

    fn insert(&mut self, v: &[usize]) {
        if self.0.insert(v, true) {
            self.1 += 1;
        }
    }

    fn len(&self) -> usize {
        self.1
    }
}

pub struct TrieCounter(Trie<u64>, usize);

impl TrieCounter {
    pub fn add(&mut self, v: &[usize], c: u64) -> bool {
        self.0.insert(v, c)
    }
}

impl Convertable for TrieCounter {
    fn to_vec(&mut self) -> Vec<u64> {
        let mut v = Vec::new();
        self.0.to_vec(&mut v);
        v
    }

    fn add_from_vec(&mut self, v: &[u64]) {
        self.0.add_from_vec(v);
    }
}

impl AprioriCounterMut for TrieCounter {
    fn for_each_mut(&mut self, mut f: impl FnMut(&[usize], &mut u64)) {
        self.0.for_each_mut(|v, b| {
            f(v, b);
        });
    }
}

impl Default for TrieCounter {
    fn default() -> Self {
        Self(Trie::new(0), 0)
    }
}

impl TrieCounter {
    pub fn new() -> Self {
        Self::default()
    }
}
impl AprioriCounting for TrieCounter {
    fn count_fn(&mut self, v: &[usize], n: usize, f: impl FnMut(&[usize])) {
        self.0.count_fn(v, n, f);
    }
}
impl AprioriCounter for TrieCounter {
    fn len(&self) -> usize {
        self.1
    }
    fn increment(&mut self, v: &[usize]) -> bool {
        self.0.increment(v)
    }

    fn insert(&mut self, v: &[usize]) {
        self.0.insert(v, 0);
        self.1 += 1;
    }

    fn get_count(&self, v: &[usize]) -> Option<u64> {
        self.0.get(v)
    }

    fn for_each(&self, mut f: impl FnMut(&[usize], u64)) {
        self.0.for_each(|v, c| {
            f(v, c);
        });
    }
}

pub struct AprioriTransition(Trie<(usize, u64)>);
impl AprioriTransition {
    pub fn new() -> Self {
        Self(Trie::new((usize::MAX, 0)))
    }
    pub fn count_fn(&mut self, v: &[usize], n: usize, mut f: impl FnMut(usize)) {
        let mut stack = Vec::new();
        self.0.count_fn_helper(v, &mut stack, n, &mut |_, i| {
            i.1 += 1;
            f(i.0)
        });
    }
    pub fn insert(&mut self, v: &[usize], i: usize) {
        self.0.insert(v, (i, 0));
    }
    pub fn for_each(&self, f: impl FnMut(&[usize], (usize, u64))) {
        self.0.for_each(f);
    }
}

impl Default for AprioriTransition {
    fn default() -> Self {
        Self::new()
    }
}
pub struct Trie<T> {
    root: TrieNode<T>,
}

impl<T> Trie<T> {
    pub fn new(value: T) -> Self {
        Self {
            root: TrieNode::new(value),
        }
    }
}

impl<T> DerefMut for Trie<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}

impl<T> Deref for Trie<T> {
    type Target = TrieNode<T>;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}
pub struct TrieNode<T> {
    children: FnvHashMap<usize, Box<TrieNode<T>>>,
    value: T,
}

impl<T: Copy + Default> TrieNode<T> {
    pub fn insert(&mut self, v: &[usize], value: T) -> bool {
        if v.is_empty() {
            self.value = value;
            return false;
        }
        if let Some(curr) = self.children.get_mut(&v[0]) {
            curr.insert(&v[1..], value)
        } else {
            let mut new = Self::new(T::default());
            new.insert(&v[1..], value);
            self.children.insert(v[0], Box::new(new));
            true
        }
    }
}

impl<T: Copy> TrieNode<T> {
    pub fn for_each(&self, mut f: impl FnMut(&[usize], T)) {
        let mut stack = vec![];
        self.for_each_helper(&mut stack, &mut f);
    }
    fn for_each_helper(&self, stack: &mut Vec<usize>, f: &mut impl FnMut(&[usize], T)) {
        f(stack, self.value);
        for (&k, v) in &self.children {
            stack.push(k);
            v.for_each_helper(stack, f);
            stack.pop();
        }
    }
    pub fn for_each_mut(&mut self, mut f: impl FnMut(&[usize], &mut T)) {
        let mut stack = vec![];
        self.for_each_mut_helper(&mut stack, &mut f);
    }
    fn for_each_mut_helper(
        &mut self,
        stack: &mut Vec<usize>,
        f: &mut impl FnMut(&[usize], &mut T),
    ) {
        f(stack, &mut self.value);
        for (&k, v) in &mut self.children {
            stack.push(k);
            v.for_each_mut_helper(stack, f);
            stack.pop();
        }
    }
    pub fn get(&self, v: &[usize]) -> Option<T> {
        if v.is_empty() {
            return Some(self.value);
        }
        if let Some(curr) = self.children.get(&v[0]) {
            curr.get(&v[1..])
        } else {
            None
        }
    }
    pub fn contains(&self, v: &[usize]) -> bool {
        self.get(v).is_some()
    }
}

impl TrieNode<bool> {
    pub fn to_vec(&self, v: &mut Vec<u64>) {
        if self.children.is_empty() {
            if v.is_empty() {
                v.push(u64::MAX);
                return;
            }
            let len = v.len();
            let before = v[len - 1];
            v[len - 1] = before
                .checked_add(1 << 63)
                .expect("Overflow occurred in converting to vector due to item id being > 2^63");
            return;
        }
        v.push(self.children.len() as u64);
        for (&k, child) in self.children.iter() {
            v.push(k as u64);
            child.to_vec(v);
        }
    }
    pub fn add_from_vec(&mut self, v: &[u64]) {
        self.add_from_vec_helper(&mut v.iter().cloned());
    }
    fn add_from_vec_helper(&mut self, v: &mut impl Iterator<Item = u64>) {
        let size = v.next().unwrap();
        for _ in 0..size {
            let mut next = v.next().unwrap();
            let mut is_end = false;
            if next >= 1 << 63 {
                is_end = true;
                next -= 1 << 63;
            }
            match self.children.get_mut(&(next as usize)) {
                Some(child) => {
                    child.value = is_end || child.value;
                    if !is_end {
                        child.add_from_vec_helper(v);
                    }
                }
                None => {
                    let mut child = TrieNode::new(is_end);
                    if !is_end {
                        child.add_from_vec_helper(v);
                    }
                    self.children.insert(next as usize, Box::new(child));
                }
            };
        }
    }
}
impl AprioriCounting for TrieNode<u64> {
    fn count_fn(&mut self, v: &[usize], n: usize, mut f: impl FnMut(&[usize])) {
        let mut vec = Vec::new();
        self.count_fn_helper(v, &mut vec, n, &mut |v, c| {
            *c += 1;
            f(v);
        });
    }
}
impl TrieNode<u64> {
    pub fn increment(&mut self, v: &[usize]) -> bool {
        if v.is_empty() {
            self.value += 1;
            return true;
        }
        if let Some(curr) = self.children.get_mut(&v[0]) {
            curr.increment(&v[1..])
        } else {
            false
        }
    }
    pub fn to_vec(&self, v: &mut Vec<u64>) {
        if self.children.is_empty() {
            v.push(
                self.value
                    .checked_add(1 << 63)
                    .expect("Overflow occurred in converting to vector due to count being > 2^63"),
            );
            return;
        }
        self.for_each(|_, c| {
            v.push(c);
        });
    }
    pub fn add_from_vec(&mut self, v: &[u64]) {
        self.add_from_vec_helper(&mut v.iter().cloned());
    }
    fn add_from_vec_helper(&mut self, v: &mut impl Iterator<Item = u64>) {
        self.for_each_mut(|_, c| {
            *c += v.next().expect("Transfer failed due to tree being larger");
        });
        assert!(
            v.next().is_none(),
            "Transfer failed due to vector being larger"
        )
    }
}
impl<T> TrieNode<T> {
    fn new(value: T) -> Self {
        Self {
            children: FnvHashMap::default(),
            value,
        }
    }
    fn count_fn_helper(
        &mut self,
        v: &[usize],
        curr: &mut Vec<usize>,
        ind: usize,
        f: &mut impl FnMut(&[usize], &mut T),
    ) {
        if ind == 0 {
            f(curr, &mut self.value);
            return;
        }
        if v.len() < ind {
            return;
        }
        for (i, &n) in v.iter().enumerate() {
            if let Some(node) = self.children.get_mut(&n) {
                curr.push(n);
                node.count_fn_helper(&v[(i + 1)..], curr, ind - 1, f);
                curr.pop();
            }
        }
    }
}
impl<T: Copy + Eq> TrieNode<T> {
    pub fn cleanup(&mut self, empty: T) {
        self.cleanup_helper(empty);
    }
    fn cleanup_helper(&mut self, empty: T) -> bool {
        self.children.retain(|_, v| !v.cleanup_helper(empty));
        self.children.is_empty() && self.value == empty
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::storage::{AprioriCounter, AprioriFrequent};

    use super::{Trie, TrieCounter, TrieSet};

    #[test]
    fn test_counter_trie() {
        let mut trie = TrieCounter::new();
        assert!(!trie.increment(&[0]));
        trie.insert(&[1, 2]);
        assert_eq!(trie.get_count(&[1]), Some(0));
        assert_eq!(trie.get_count(&[1, 2]), Some(0));
        assert_eq!(trie.get_count(&[1, 2, 3]), None);
        trie.increment(&[1, 2, 3]);
        assert_eq!(trie.get_count(&[1]), Some(0));
        assert_eq!(trie.get_count(&[1, 2]), Some(0));
        assert_eq!(trie.get_count(&[1, 2, 3]), None);
        trie.increment(&[1, 2]);
        assert_eq!(trie.get_count(&[1]), Some(0));
        assert_eq!(trie.get_count(&[1, 2]), Some(1));
    }
    #[test]
    fn test_trie_set() {
        let mut trie = TrieSet::new();
        assert!(trie.is_empty());
        assert!(trie.len() == 0);
        assert!(!trie.contains(&[0]));
        trie.insert(&[1, 2]);
        assert!(!trie.is_empty());
        assert!(trie.len() == 1);
        assert!(trie.contains(&[1, 2]));
        assert!(!trie.contains(&[1]));
        trie.insert(&[1, 3]);
        let mut set = HashSet::new();
        set.insert(vec![1, 2]);
        set.insert(vec![1, 3]);
        trie.for_each(|v| {
            assert!(set.remove(v));
        });
        assert!(set.is_empty());
    }
    #[test]
    fn test_cleanup() {
        let mut trie = Trie::new(0u64);
        trie.insert(&[1, 2, 3], 0);
        trie.insert(&[1, 2, 4], 1);
        trie.cleanup(0);
        assert!(!trie.contains(&[1, 2, 3]));
        assert!(trie.contains(&[1, 2, 4]));
    }
    #[test]
    fn test_convertable() {
        let mut trie = Trie::new(0u64);
        trie.insert(&[1, 2, 3], 2);
        trie.insert(&[1, 2, 4], 5);
        trie.insert(&[1, 3, 4], 6);
        let mut v = Vec::new();
        trie.to_vec(&mut v);
        let mut trie = Trie::new(0u64);
        trie.insert(&[1, 2, 3], 0);
        trie.insert(&[1, 2, 4], 0);
        trie.insert(&[1, 3, 4], 0);
        trie.add_from_vec(&v);
        assert_eq!(trie.get(&[1, 2, 3]), Some(2));
        assert_eq!(trie.get(&[1, 2, 4]), Some(5));
        assert_eq!(trie.get(&[1, 3, 4]), Some(6));

        let mut trie2 = Trie::new(0u64);
        trie2.insert(&[1, 2, 3], 2);
        trie2.insert(&[1, 2, 4], 5);
        trie2.insert(&[1, 3, 4], 6);

        trie2.insert(&[1, 3, 5], 8);
        trie.insert(&[1, 3, 5], 0);
        let mut v = Vec::new();
        trie2.to_vec(&mut v);
        trie.add_from_vec(&v);
        assert_eq!(trie.get(&[1, 2, 3]), Some(4));
        assert_eq!(trie.get(&[1, 2, 4]), Some(10));
        assert_eq!(trie.get(&[1, 3, 4]), Some(12));
        assert_eq!(trie.get(&[1, 3, 5]), Some(8));

        let mut trie = Trie::new(false);
        trie.insert(&[1, 2, 3], true);
        trie.insert(&[1, 2, 4], true);
        trie.insert(&[1, 3, 4], true);
        let mut v = Vec::new();
        trie.to_vec(&mut v);
        let mut trie = Trie::new(false);
        trie.add_from_vec(&v);
        assert_eq!(trie.get(&[1, 2, 3]), Some(true));
        assert_eq!(trie.get(&[1, 2, 4]), Some(true));
        assert_eq!(trie.get(&[1, 3, 4]), Some(true));
    }
}
