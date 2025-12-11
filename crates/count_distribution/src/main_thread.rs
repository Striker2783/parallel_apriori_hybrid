use std::{sync::{Arc, Mutex}, time::Instant};

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

pub(crate) struct MainRunner<'a> {
    sup: u64,
    writer: Arc<Mutex<dyn Write + 'a>>,
    uni: &'a Universe,
    counter: Arc<Mutex<dyn ParallelCounting + 'a>>,
}

impl<'a> MainRunner<'a> {
    pub fn new(sup: u64, writer: Arc<Mutex<dyn Write>>, uni: &'a Universe, counter: Arc<Mutex<dyn ParallelCounting>>) -> Self {
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
        combined.add_from_vec(&self.counter.lock().unwrap().count_2(p1));
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
        let mut writer = self.writer.lock().unwrap();
        p.for_each(|v| writer.write_set(v));
        println!("2 {:?}", prev_time.elapsed());
        if p.is_empty() {
            drop(writer);
            self.end();
            return;
        }
        for i in 3.. {
            let prev_time = Instant::now();
            let converted = p.to_vec();
            for i in 1..self.uni.world().size() {
                self.uni.world().process_at_rank(i).send(&converted);
            }
            let mut counter = self.counter.lock().unwrap();
            counter.count(&p, i);
            for _ in 1..self.uni.world().size() {
                let (v, _) = self.uni.world().any_process().receive_vec();
                counter.add(&v);
            }
            p = counter.frequent(self.sup);
            println!("{i} {:?}", prev_time.elapsed());
            if p.is_empty() {
                break;
            }
            let mut writer = self.writer.lock().unwrap();
            p.for_each(|v| {
                writer.write_set(v);
            });
        }
        drop(writer);
        self.end();
    }
    pub fn preprocess(&mut self, data: &TransactionSet) -> Vec<usize> {
        let p = apriori_pass_one(data, self.sup);
        let mut writer = self.writer.lock().unwrap();
        p.iter().for_each(|&n| writer.write_set(&[n]));
        p
    }
}
