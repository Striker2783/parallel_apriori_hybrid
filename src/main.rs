use apriori::apriori::AprioriRunner;
use apriori::start::{Apriori, Write};
use apriori::transaction_set::TransactionSet;
use apriori_tid::hybrid::AprioriHybrid;
use apriori_tid::tid::AprioriTIDRunner;
use clap::Parser;
use clap::*;
use count_distribution::runner::CountDistribution;
use parallel::traits::ParallelRun;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser)]
pub struct Args {
    file: PathBuf,
    support_count: u64,
    algorithm: Algorithms,
    #[arg(short, long, default_value = "false")]
    time: bool,
    #[arg(short, long)]
    output: Option<PathBuf>,
}
#[derive(Debug, Clone, ValueEnum)]
pub enum Algorithms {
    Apriori,
    CountDistribution,
    AprioriTID,
    AprioriHybrid,
}

pub struct Inputs<T: Write> {
    data: TransactionSet,
    support_count: u64,
    out: T,
}

impl<T: Write> Inputs<T> {
    pub fn new(data: TransactionSet, support_count: u64, out: T) -> Self {
        Self {
            data,
            support_count,
            out,
        }
    }
}
#[derive(Default)]
pub struct EmptyWriter();
impl EmptyWriter {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Write for EmptyWriter {
    fn write_set(&mut self, _: &[usize]) {}
}
#[derive(Debug)]
pub enum MainError {
    InvalidInputFile(std::io::Error),
    InvalidOutputFile(std::io::Error),
}

fn aa<T: Write>(mut input: Inputs<T>, v: &Args) {
    match v.algorithm {
        Algorithms::Apriori => {
            let runner = AprioriRunner::new(&input.data, input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::CountDistribution => {
            let runner = CountDistribution::new(&input.data, input.support_count, &mut input.out);
            runner.run();
        }
        Algorithms::AprioriTID => {
            let runner = AprioriTIDRunner::new(&input.data, input.support_count);
            runner.run(&mut input.out);
        },
        Algorithms::AprioriHybrid => {
            let runner = AprioriHybrid::new(&input.data, input.support_count);
            runner.run(&mut input.out);
        }
    }
}

fn main() -> Result<(), MainError> {
    let a = Args::parse();
    let file = File::open(&a.file).map_err(MainError::InvalidInputFile)?;
    let data = TransactionSet::from_dat(file);
    let before = Instant::now();
    match &a.output {
        Some(f) => {
            let out = File::create(f).map_err(MainError::InvalidOutputFile)?;
            let writer = BufWriter::new(out);
            let input = Inputs::new(data, a.support_count, writer);
            aa(input, &a);
        }
        None => {
            let input = Inputs::new(data, a.support_count, EmptyWriter::new());
            aa(input, &a);
        }
    };
    println!("Time Taken: {:?}", before.elapsed());
    Ok(())
}
