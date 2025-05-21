use crate::storage::AprioriFrequent;

pub trait AprioriOne<T: AprioriFrequent> {
    fn run_one(self) -> T;
}
pub trait AprioriTwo<T: AprioriFrequent> {
    fn run_two(self) -> T;
}

pub trait AprioriGeneral<T: AprioriFrequent> {
    fn run(self, counter: &impl AprioriFrequent, n: usize) -> T;
}

pub trait Apriori {
    fn run<T: Write>(self, out: &mut T);
}

pub trait Write {
    fn write_set(&mut self, v: &[usize]);
}
impl<T: std::io::Write> Write for T {
    fn write_set(&mut self, v: &[usize]) {
        let mut s = String::new();
        for &n in v {
            s += format!("{n} ").as_str();
        }
        s += "\n";
        let _ = self.write(s.as_bytes());
    }
}

pub struct FrequentWriter<T: AprioriFrequent> {
    inner: T,
}

impl<T: AprioriFrequent> FrequentWriter<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: AprioriFrequent> Write for FrequentWriter<T> {
    fn write_set(&mut self, v: &[usize]) {
        self.inner.insert(v);
    }
}

impl<T: AprioriFrequent + Default> FrequentWriter<T> {
    pub fn new() -> Self {
        Self {
            inner: T::default(),
        }
    }
}

impl<T: AprioriFrequent + Default> Default for FrequentWriter<T> {
    fn default() -> Self {
        Self::new()
    }
}
