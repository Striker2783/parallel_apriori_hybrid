use apriori::{
    apriori::AprioriPass1And2,
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
            let b = a.preprocess(self.data);
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
        for n in 3.. {
            let a: (Vec<u64>, mpi::point_to_point::Status) =
                self.uni.world().process_at_rank(0).receive_vec();
            if a.0[0] == u64::MAX {
                break;
            }
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

struct MainRunner<'a, T: Write> {
    sup: u64,
    writer: &'a mut T,
    uni: Universe,
}

impl<'a, T: Write> MainRunner<'a, T> {
    fn new(sup: u64, writer: &'a mut T, uni: Universe) -> Self {
        Self { sup, writer, uni }
    }

    fn run(&mut self, mut p: TrieSet) {
        for _ in 3.. {
            let converted = p.to_vec();
            for i in 1..self.uni.world().size() {
                self.uni.world().process_at_rank(i).send(&converted);
            }
            let mut combined = TrieCounter::new();
            for _ in 1..self.uni.world().size() {
                let (v, _) = self.uni.world().any_process().receive_vec();
                combined.add_from_vec(&v);
            }
            p = combined.to_frequent_new(self.sup);
            if p.is_empty() {
                break;
            }
            p.for_each(|v| {
                self.writer.write_set(v);
            });
        }
        for i in 1..self.uni.world().size() {
            self.uni.world().process_at_rank(i).send(&[u64::MAX]);
        }
    }
    fn preprocess(&mut self, data: &TransactionSet) -> TrieSet {
        AprioriPass1And2::new(self.sup, data).run(self.writer)
    }
}
