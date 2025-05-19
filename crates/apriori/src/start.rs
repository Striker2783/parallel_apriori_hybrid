use crate::storage::{Counter, Frequent};

pub trait AprioriOne {
    fn run_one(self) -> impl Frequent;
}
pub trait AprioriTwo {
    fn run_two(self) -> impl Frequent;
}

pub trait AprioriCounter {
    fn run(self) -> impl Counter;
}