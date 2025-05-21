use std::ops::{Deref, DerefMut};

use ahash::AHashMap;

use crate::storage::{AprioriCounter, AprioriCounterMut, AprioriFrequent};

pub struct TrieSet(Trie<bool>, usize);

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

impl AprioriCounterMut for TrieCounter {
    fn for_each_mut(&mut self, mut f: impl FnMut(&[usize], &mut u64)) {
        self.0.for_each_mut(|v, b| {
            f(v, b);
        });
    }
    fn len(&self) -> usize {
        self.1
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
impl AprioriCounter for TrieCounter {
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
    children: AHashMap<usize, Box<TrieNode<T>>>,
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
}

impl<T> TrieNode<T> {
    fn new(value: T) -> Self {
        Self {
            children: AHashMap::new(),
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::storage::{AprioriCounter, AprioriFrequent};

    use super::{TrieCounter, TrieSet};

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
}
