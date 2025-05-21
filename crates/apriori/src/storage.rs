use ahash::AHashMap;

pub trait AprioriCounter {
    fn increment(&mut self, v: &[usize]) -> bool;
    fn insert(&mut self, v: &[usize]);
    fn get_count(&self, v: &[usize]) -> Option<u64>;

    fn contains(&self, v: &[usize]) -> bool {
        self.get_count(v).is_some()
    }
    fn for_each(&self, f: impl FnMut(&[usize], u64));
    fn to_frequent_new<T>(&self, sup: u64) -> T
    where
        T: AprioriFrequent + Default,
    {
        let mut f = T::default();
        self.to_frequent(&mut f, sup);
        f
    }
    fn to_frequent<T: AprioriFrequent>(&self, set: &mut T, sup: u64) {
        self.for_each(|v, count| {
            if count >= sup {
                set.insert(v);
            }
        });
    }
}
pub trait AprioriCounterMut: AprioriCounter {
    fn for_each_mut(&mut self, f: impl FnMut(&[usize], &mut u64));
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl AprioriCounter for Vec<u64> {
    fn increment(&mut self, v: &[usize]) -> bool {
        self[v[0]] += 1;
        true
    }

    fn insert(&mut self, _: &[usize]) {
        panic!("Should alreadby be inserted");
    }

    fn get_count(&self, v: &[usize]) -> Option<u64> {
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

pub trait AprioriFrequent {
    fn for_each(&self, f: impl FnMut(&[usize]));
    fn contains(&self, v: &[usize]) -> bool;
    fn insert(&mut self, v: &[usize]);
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn join_new<T: AprioriCounter + Default>(&self) -> T {
        let mut f = T::default();
        self.join(&mut f);
        f
    }
    fn join<T: AprioriCounter>(&self, counter: &mut T) {
        let mut map: AHashMap<Vec<usize>, Vec<usize>> = AHashMap::new();
        self.for_each(|v| {
            match map.get_mut(&v[..(v.len() - 1)]) {
                Some(vec) => vec.push(*v.last().unwrap()),
                None => {
                    map.insert(v[..(v.len() - 1)].to_vec(), vec![*v.last().unwrap()]);
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

impl AprioriFrequent for Vec<bool> {
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
        storage::AprioriCounter,
        trie::{TrieCounter, TrieSet},
    };

    use super::AprioriFrequent;

    #[test]
    fn test_join() {
        let mut trie = TrieSet::new();
        trie.insert(&[1, 2]);
        trie.insert(&[1, 3]);
        trie.insert(&[2, 3]);
        let join = trie.join_new::<TrieCounter>();
        assert_eq!(join.get_count(&[1, 2]), Some(0));
        assert_eq!(join.get_count(&[1, 2, 3]), Some(0));
        assert_eq!(join.get_count(&[2, 3]), None);
    }
}
