use std::time::Instant;

use apriori::{
    apriori::apriori_pass_one,
    array2d::AprioriP2Counter,
    start::Write,
    storage::{AprioriCounter, AprioriFrequent},
    transaction_set::TransactionSet,
    trie::TrieSet,
};
use mpi::{
    environment::Universe,
    traits::{Communicator, Destination, Source},
};
use parallel::traits::Convertable;

pub trait ParallelCounting {
    fn count(&mut self, set: &TrieSet, n: usize) -> Vec<u64>;
    fn add(&mut self, v: &[u64]);
    fn count_2(&mut self, prev: &[usize]) -> Vec<u64>;
    fn frequent(&mut self, sup: u64) -> TrieSet;
}

pub(crate) struct MainRunner<'a, T: Write, U: ParallelCounting> {
    sup: u64,
    writer: &'a mut T,
    uni: &'a Universe,
    counter: U,
}

impl<'a, T: Write, U: ParallelCounting> MainRunner<'a, T, U> {
    pub fn new(sup: u64, writer: &'a mut T, uni: &'a Universe, counter: U) -> Self {
        Self {
            sup,
            writer,
            uni,
            counter,
        }
    }
    fn end(&mut self) {
        for i in 1..self.uni.world().size() {
            self.uni.world().process_at_rank(i).send(&[u64::MAX]);
        }
    }
    fn pass_two(&mut self, p1: &[usize]) -> TrieSet {
        let p1set: Vec<u64> = p1.iter().map(|&n| n as u64).collect();
        for i in 1..self.uni.world().size() {
            self.uni.world().process_at_rank(i).send(&p1set);
        }
        let mut combined = AprioriP2Counter::new(p1);
        combined.add_from_vec(&self.counter.count_2(p1));
        for _ in 1..self.uni.world().size() {
            let (v, _) = self.uni.world().any_process().receive_vec();
            combined.add_from_vec(&v);
        }
        combined.to_frequent_new(self.sup)
    }
    pub fn run(&mut self, p1: Vec<usize>) {
        if p1.is_empty() {
            self.end();
            return;
        }
        let prev_time = Instant::now();
        let mut p = self.pass_two(&p1);
        p.for_each(|v| self.writer.write_set(v));
        println!("2 {:?}", prev_time.elapsed());
        if p.is_empty() {
            self.end();
            return;
        }
        for i in 3.. {
            let prev_time = Instant::now();
            let converted = p.to_vec();
            for i in 1..self.uni.world().size() {
                self.uni.world().process_at_rank(i).send(&converted);
            }
            let mut main_p = TrieSet::new();
            main_p.add_from_vec(&converted);
            self.counter.count(&main_p, i);
            for _ in 1..self.uni.world().size() {
                let (v, _) = self.uni.world().any_process().receive_vec();
                self.counter.add(&v);
            }
            p = self.counter.frequent(self.sup);
            println!("{i} {:?}", prev_time.elapsed());
            if p.is_empty() {
                break;
            }
            p.for_each(|v| {
                self.writer.write_set(v);
            });
        }
        self.end();
    }
    pub fn preprocess(&mut self, data: &TransactionSet) -> Vec<usize> {
        let p = apriori_pass_one(data, self.sup);
        p.iter().for_each(|&n| self.writer.write_set(&[n]));
        p
    }
}
