use std::time::Instant;

use apriori::{
    array2d::AprioriP2Counter2,
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

use crate::main_thread::MainRunner;

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
    fn run(self) {
        let universe = mpi::initialize().unwrap();
        let size = universe.world().size();
        if size < 2 {
            panic!("Rank must be at least 2")
        }
        let rank = universe.world().rank();
        if rank == 0 {
            let mut a = MainRunner::new(self.sup, self.writer, universe);
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

struct HelperRunner {
    data: TransactionSet,
    uni: Universe,
}

impl HelperRunner {
    pub fn new(data: &TransactionSet, uni: Universe) -> Self {
        let world = uni.world();
        let count = data.len() / (world.size() - 1) as usize;
        let thread = world.rank() as usize - 1;
        let slice = if world.rank() == world.size() - 1 {
            &data.transactions[(count * thread)..data.len()]
        } else {
            &data[(count * thread)..(count * (thread + 1))]
        };
        let data = TransactionSet::new(slice.to_vec(), data.num_items);

        Self { data, uni }
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
                let mut counter = AprioriP2Counter2::new(&a);
                for d in self.data.iter() {
                    for (i, a) in d.iter().cloned().enumerate() {
                        for b in d.iter().cloned().skip(i + 1) {
                            counter.increment(&[a, b]);
                        }
                    }
                }
                let v = counter.to_vec();
                self.uni.world().process_at_rank(0).send(&v);
            } else {
                let mut trie = TrieSet::new();
                trie.add_from_vec(&a.0);
                let mut counter: TrieCounter = trie.join_new();
                for d in self.data.iter() {
                    counter.count(d, n);
                }
                let v = counter.to_vec();
                self.uni.world().process_at_rank(0).send(&v);
            }
        }
    }
}

