use std::io::Write;

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