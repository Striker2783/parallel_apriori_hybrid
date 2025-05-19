use crate::storage::AprioriFrequent;

pub trait AprioriOne {
    fn run_one(self) -> impl AprioriFrequent;
}
pub trait AprioriTwo {
    fn run_two(self) -> impl AprioriFrequent;
}

pub trait AprioriGeneral {
    fn run(self, counter: &impl AprioriFrequent, n: usize) -> impl AprioriFrequent;
}
