use crate::{storage::AprioriCounterMut, transaction_set::TransactionSet};

pub trait Count {
    fn count(self, n: usize);
}

pub struct AprioriCounting<'a, T: AprioriCounterMut> {
    data: &'a TransactionSet,
    counter: &'a mut T,
}

impl<'a, T: AprioriCounterMut> AprioriCounting<'a, T> {
    pub fn new(data: &'a TransactionSet, counter: &'a mut T) -> Self {
        Self { data, counter }
    }
}

impl<T: AprioriCounterMut> Count for AprioriCounting<'_, T> {
    fn count(self, n: usize) {
        for d in self.data.iter() {
            if d.len() < n {
                continue;
            }
            let mut combinations =
                ((d.len() - n + 1).max(n + 1)..=d.len()).fold(1f64, |acc, x| acc * (x as f64));
            if combinations.is_finite() {
                combinations /= (2..(d.len() - n + 1).min(n + 1)).fold(1f64, |a, n| a * (n as f64));
            }
            if (self.counter.len() as f64) * (n as f64) > combinations {
                let mut c = Combinations::new(n, d);
                c.combinations(|v| {
                    self.counter.increment(v);
                });
            } else {
                self.counter.for_each_mut(|v, c| {
                    if v.len() < n {
                        return;
                    }
                    let mut iter = d.iter().cloned();
                    'outer: for &a in v {
                        for b in iter.by_ref() {
                            match a.cmp(&b) {
                                std::cmp::Ordering::Less => return,
                                std::cmp::Ordering::Equal => continue 'outer,
                                std::cmp::Ordering::Greater => continue,
                            }
                        }
                        return;
                    }
                    *c += 1;
                });
            }
        }
    }
}

struct Combinations<'a> {
    stack: Vec<usize>,
    data: &'a [usize],
}

impl<'a> Combinations<'a> {
    fn new(n: usize, data: &'a [usize]) -> Self {
        Self {
            stack: vec![0; n],
            data,
        }
    }
    fn combinations(&mut self, mut f: impl FnMut(&[usize])) {
        self.combinations_helper(0, 0, &mut f);
    }
    fn combinations_helper(&mut self, i: usize, start: usize, f: &mut impl FnMut(&[usize])) {
        if i >= self.stack.len() {
            f(&self.stack);
            return;
        }
        for j in start..self.data.len() {
            self.stack[i] = self.data[j];
            self.combinations_helper(i + 1, j + 1, f);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::Combinations;

    #[test]
    fn test_combinations() {
        let mut c = Combinations::new(2, &[0, 1, 2, 3]);
        let mut set = HashSet::new();
        set.insert([0, 1]);
        set.insert([0, 2]);
        set.insert([0, 3]);
        set.insert([1, 2]);
        set.insert([1, 3]);
        set.insert([2, 3]);
        c.combinations(|v| assert!(set.remove(v)));
        assert!(set.is_empty());
    }
}
