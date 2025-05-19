use crate::storage::{AprioriCounter, AprioriFrequent};

pub trait AprioriOne {
    fn run_one(self) -> impl AprioriFrequent;
}
pub trait AprioriTwo {
    fn run_two(self) -> impl AprioriFrequent;
}

pub trait Apriori {
    fn run(self) -> impl AprioriCounter;
}