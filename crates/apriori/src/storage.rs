use ahash::AHashMap;

pub trait Counter {
    fn increment(&mut self, v: &[usize]) -> bool;
    fn insert(&mut self, v: &[usize]);
    fn get(&self, v: &[usize]) -> Option<u64>;

    fn contains(&self, v: &[usize]) -> bool {
        self.get(v).is_some()
    }
    fn for_each(&self, f: impl FnMut(&[usize], u64));
    fn to_frequent_new<T>(&self, sup: u64) -> T
    where
        T: Frequent + Default,
    {
        let mut f = T::default();
        self.to_frequent(&mut f, sup);
        f
    }
    fn to_frequent<T: Frequent>(&self, set: &mut T, sup: u64) {
        self.for_each(|v, count| {
            if count >= sup {
                set.insert(v);
            }
        });
    }
}
impl Counter for Vec<u64> {
    fn increment(&mut self, v: &[usize]) -> bool {
        self[v[0]] += 1;
        true
    }

    fn insert(&mut self, v: &[usize]) {
        panic!("Should alreadby be inserted");
    }

    fn get(&self, v: &[usize]) -> Option<u64> {
        Some(self[v[0]])
    }

    fn for_each(&self, mut f: impl FnMut(&[usize], u64)) {
        let mut v = [0usize];
        self.iter().enumerate().for_each(|(i, &c)| {
            v[0] = i;
            f(&v, c);
        });
    }
}

pub trait Frequent {
    fn for_each(&self, f: impl FnMut(&[usize]));
    fn contains(&self, v: &[usize]) -> bool;
    fn insert(&mut self, v: &[usize]);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn join_new<T: Counter + Default>(&self) -> T {
        let mut f = T::default();
        self.join(&mut f);
        f
    }
    fn join<T: Counter>(&self, counter: &mut T) {
        let mut map: AHashMap<Vec<usize>, Vec<usize>> = AHashMap::new();
        self.for_each(|v| {
            match map.get_mut(&v[..(v.len() - 1)]) {
                Some(vec) => vec.push(v.last().unwrap().clone()),
                None => {
                    map.insert(v[..(v.len() - 1)].to_vec(), vec![v.last().unwrap().clone()]);
                }
            };
        });
        for (mut prefix, suffix) in map {
            for (i, last1) in suffix.iter().cloned().enumerate() {
                for last2 in suffix.iter().skip(i + 1).cloned() {
                    let (min, max) = (last1.min(last2), last1.max(last2));
                    prefix.push(min);
                    prefix.push(max);
                    counter.insert(&prefix);
                    prefix.pop();
                    prefix.pop();
                }
            }
        }
    }
}

impl Frequent for Vec<bool> {
    fn for_each(&self, mut f: impl FnMut(&[usize])) {
        let mut v = [0];
        self.iter().enumerate().for_each(|(i, &b)| {
            if !b {
                return;
            }
            v[0] = i;
            f(&v);
        });
    }

    fn contains(&self, v: &[usize]) -> bool {
        self.get(v[0]).copied().unwrap_or(false)
    }

    fn insert(&mut self, v: &[usize]) {
        self[v[0]] = true;
    }

    fn len(&self) -> usize {
        self.iter().filter(|b| **b).count()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        storage::Counter,
        trie::{TrieCounter, TrieSet},
    };

    use super::Frequent;

    #[test]
    fn test_join() {
        let mut trie = TrieSet::new();
        trie.insert(&[1, 2]);
        trie.insert(&[1, 3]);
        trie.insert(&[2, 3]);
        let join = trie.join_new::<TrieCounter>();
        assert_eq!(join.get(&[1, 2]), Some(0));
        assert_eq!(join.get(&[1, 2, 3]), Some(0));
        assert_eq!(join.get(&[2, 3]), None);
    }
}
