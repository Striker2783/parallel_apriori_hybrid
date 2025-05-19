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
