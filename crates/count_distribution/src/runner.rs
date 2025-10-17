use std::{path::Path, time::Instant};

use apriori::{
    apriori::apriori_pass_two_counter,
    array2d::AprioriP2Counter,
    start::Write,
    storage::{AprioriCounter, AprioriCounting, AprioriFrequent},
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
    data: &'a Path,
    sup: u64,
    writer: &'a mut T,
}

impl<'a, T: Write> CountDistribution<'a, T> {
    pub fn new(data: &'a Path, sup: u64, writer: &'a mut T) -> Self {
        Self { data, sup, writer }
    }
}

impl<T: Write> ParallelRun for CountDistribution<'_, T> {
    fn run(self, universe: &Universe) {
        let size = universe.world().size();
        assert!(size > 1, "Rank must be at least 2");
        let rank = universe.world().rank();
        if rank == 0 {
            let data = TransactionSet::from_path(self.data).unwrap();
            let world = universe.world();

            let mut data_iter = data.partition(world.size() as usize);
            let first_data = data_iter.next().unwrap();

            for i in 1..world.size() {
                world
                    .process_at_rank(i)
                    .send(&data_iter.next().unwrap().to_vec());
            }

            let mut a = MainRunner::new(
                self.sup,
                self.writer,
                universe,
                MainHelper::new_from_transaction_set(first_data),
            );
            let temp = Instant::now();
            let b = a.preprocess(&data);
            println!("Preprocess {:?}", temp.elapsed());
            a.run(b);
        } else {
            let mut a = HelperRunner::new(universe);
            a.run();
        }
    }
}

struct MainHelper {
    data: TransactionSet,
    counter: TrieCounter,
}
impl MainHelper {
    pub fn new(uni: &Universe) -> Self {
        let world = uni.world();

        let mut data = TransactionSet::default();
        let received: (Vec<u64>, mpi::point_to_point::Status) =
            world.process_at_rank(0).receive_vec();
        data.add_from_vec(&received.0);

        Self {
            data,
            counter: TrieCounter::new(),
        }
    }
    pub fn new_from_transaction_set(data: TransactionSet) -> Self {
        Self {
            data,
            counter: TrieCounter::new(),
        }
    }
}
impl ParallelCounting for MainHelper {
    fn count(&mut self, set: &TrieSet, n: usize) -> Vec<u64> {
        let mut counter: TrieCounter = set.join_new();
        for d in self.data.iter() {
            counter.count(d, n);
        }
        let v = counter.to_vec();
        self.counter = counter;
        v
    }

    fn count_2(&mut self, prev: &[usize]) -> Vec<u64> {
        let mut p2 = AprioriP2Counter::new(prev);
        apriori_pass_two_counter(&self.data, &mut p2);
        p2.to_vec()
    }

    fn add(&mut self, v: &[u64]) {
        self.counter.add_from_vec(v);
    }

    fn frequent(&mut self, sup: u64) -> TrieSet {
        self.counter.to_frequent_new(sup)
    }
}

struct HelperRunner<'a> {
    counter: MainHelper,
    uni: &'a Universe,
}

impl<'a> HelperRunner<'a> {
    pub fn new(uni: &'a Universe) -> Self {
        let counter = MainHelper::new(uni);
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
