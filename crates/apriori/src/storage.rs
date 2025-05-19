pub trait Counter {
    fn increment(&mut self, v: &[usize]) -> bool;
    fn insert(&mut self, v: &[usize]);
    fn get(&self, v: &[usize]) -> Option<u64>;

    fn contains(&self, v: &[usize]) -> bool {
        self.get(v).is_some()
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
