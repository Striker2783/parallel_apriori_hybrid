use std::time::Instant;

use apriori::{
    apriori::apriori_pass_two_counter,
    array2d::AprioriP2Counter,
    start::Write,
    storage::{AprioriCounting, AprioriFrequent},
    transaction_set::TransactionSet,
    trie::{TrieCounter, TrieSet},
};
use mpi::{
    environment::Universe,
    traits::{Communicator, Destination, Source},
};
use parallel::traits::{Convertable, ParallelRun};

use crate::main_thread::{MainRunner, ParallelCounting};

pub struct CountDistribution<'a, T: Write> {
    data: &'a TransactionSet,
    sup: u64,
    writer: &'a mut T,
}

impl<'a, T: Write> CountDistribution<'a, T> {
    pub fn new(data: &'a TransactionSet, sup: u64, writer: &'a mut T) -> Self {
        Self { data, sup, writer }
    }
}

impl<T: Write> ParallelRun for CountDistribution<'_, T> {
    fn run(self, universe: &Universe) {
        let size = universe.world().size();
        if size < 2 {
            panic!("Rank must be at least 2")
        }
        let rank = universe.world().rank();
        if rank == 0 {
            let mut a = MainRunner::new(
                self.sup,
                self.writer,
                universe,
                MainHelper::new(self.data, universe),
            );
            let temp = Instant::now();
            let b = a.preprocess(self.data);
            println!("Preprocess {:?}", temp.elapsed());
            a.run(b);
        } else {
            let mut a = HelperRunner::new(self.data, universe);
            a.run();
        }
    }
}

struct MainHelper {
    data: TransactionSet,
}
impl MainHelper {
    pub fn new(data: &TransactionSet, uni: &Universe) -> Self {
        let world = uni.world();
        let count = data.len() / world.size() as usize;
        let thread = world.rank() as usize;
        let slice = if world.rank() == world.size() - 1 {
            &data.transactions[(count * thread)..data.len()]
        } else {
            &data[(count * thread)..(count * (thread + 1))]
        };
        let data = TransactionSet::new(slice.to_vec(), data.num_items);

        Self { data }
    }
}
impl ParallelCounting for MainHelper {
    fn count(&mut self, set: &TrieSet, n: usize) -> Vec<u64> {
        let mut counter: TrieCounter = set.join_new();
        for d in self.data.iter() {
            counter.count(d, n);
        }
        counter.to_vec()
    }

    fn count_2(&mut self, prev: &[usize]) -> Vec<u64> {
        let mut p2 = AprioriP2Counter::new(prev);
        apriori_pass_two_counter(&self.data, &mut p2);
        p2.to_vec()
    }
}

struct HelperRunner<'a> {
    counter: MainHelper,
    uni: &'a Universe,
}

impl<'a> HelperRunner<'a> {
    pub fn new(data: &TransactionSet, uni: &'a Universe) -> Self {
        let counter = MainHelper::new(data, uni);
        Self { counter, uni }
    }
    fn run(&mut self) {
        for n in 2.. {
            let a: (Vec<u64>, mpi::point_to_point::Status) =
                self.uni.world().process_at_rank(0).receive_vec();
            if a.0[0] == u64::MAX {
                break;
            }
            if n == 2 {
                let a: Vec<_> = a.0.into_iter().map(|n| n as usize).collect();
                let v = self.counter.count_2(&a);
                self.uni.world().process_at_rank(0).send(&v);
            } else {
                let mut trie = TrieSet::new();
                trie.add_from_vec(&a.0);
                let v = self.counter.count(&trie, n);
                self.uni.world().process_at_rank(0).send(&v);
            }
        }
    }
}
