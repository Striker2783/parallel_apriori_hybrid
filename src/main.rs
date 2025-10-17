use apriori::alone::AprioriTrie;
use apriori::apriori::AprioriRunner;
use apriori::start::{Apriori, Write};
use apriori::transaction_set::TransactionSet;
use apriori_tid::hybrid::AprioriHybridRunner;
use apriori_tid::tid::AprioriTIDRunner2;
use clap::Parser;
use clap::*;
use count_distribution::hybridrunner::CountDistributionHybrid;
use count_distribution::runner::CountDistribution;
use mpi::environment::{self, Universe};
use mpi::traits::Communicator;
use parallel::traits::ParallelRun;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write as IOWrite};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

#[derive(Parser)]
pub struct Args {
    file: PathBuf,
    support_count: u64,
    algorithm: Algorithms,
    #[arg(short, long, default_value = "false")]
    time: bool,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long)]
    csv: Option<PathBuf>,
}
#[derive(Debug, Clone, ValueEnum)]
pub enum Algorithms {
    Apriori,
    CountDistribution,
    AprioriTID,
    AprioriHybrid,
    CountDistributionHybrid,
    AprioriTrie,
}

pub struct Inputs<T: Write> {
    data: PathBuf,
    support_count: u64,
    out: T,
}

impl<T: Write> Inputs<T> {
    pub fn new(data: PathBuf, support_count: u64, out: T) -> Self {
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
    InvalidOutputCSV(std::io::Error),
}

static MPI_UNIVERSE: OnceLock<Universe> = OnceLock::new();

pub fn get_universe() -> &'static Universe {
    MPI_UNIVERSE.get_or_init(|| environment::initialize().expect("Failed to initialize MPI"))
}

pub fn mpi_initialized() -> bool {
    MPI_UNIVERSE.get().is_some()
}

fn aa<T: Write>(mut input: Inputs<T>, v: &Args) -> Result<(), std::io::Error> {
    match v.algorithm {
        Algorithms::Apriori => {
            let data = TransactionSet::from_path(&input.data)?;
            let runner = AprioriRunner::new(&data, input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::CountDistribution => {
            let universe = get_universe();
            let runner = CountDistribution::new(&input.data, input.support_count, &mut input.out);
            runner.run(universe);
        }
        Algorithms::AprioriTID => {
            let data = TransactionSet::from_path(&input.data)?;
            let runner = AprioriTIDRunner2::new(&data, input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::AprioriHybrid => {
            let mut data = TransactionSet::from_path(&input.data)?;
            let runner = AprioriHybridRunner::new(&mut data, input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::CountDistributionHybrid => {
            let universe = get_universe();
            let data = TransactionSet::from_path(&input.data)?;
            let runner =
                CountDistributionHybrid::new(&data, input.support_count, &mut input.out);
            runner.run(universe);
        }
        Algorithms::AprioriTrie => {
            let runner = AprioriTrie::new(TransactionSet::from_path(&input.data)?, input.support_count);
            runner.run(&mut input.out);
        }
    }
    Ok(())
}

fn output_csv(file: &Path, duration: &Duration) -> Result<(), MainError> {
    let out = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file)
        .map_err(MainError::InvalidOutputCSV)?;
    let mut writer = BufWriter::new(out);
    let mut s = String::new();
    s.push_str(&duration.as_secs_f64().to_string());
    s.push('\n');
    let _ = IOWrite::write(&mut writer, s.as_bytes());
    Ok(())
}

fn main() -> Result<(), MainError> {
    let a = Args::parse();
    let before = Instant::now();
    match &a.output {
        Some(f) => {
            let out = File::create(f).map_err(MainError::InvalidOutputFile)?;
            let writer = BufWriter::new(out);
            let input = Inputs::new(a.file.clone(), a.support_count, writer);
            aa(input, &a).unwrap();
        }
        None => {
            let input = Inputs::new(a.file.clone(), a.support_count, EmptyWriter::new());
            aa(input, &a).unwrap();
        }
    };
    if a.time {
        println!("Time Taken: {:?}", before.elapsed());
    }
    if let Some(p) = a.csv {
        if !mpi_initialized() || (mpi_initialized() && get_universe().world().rank() == 0) {
            output_csv(
                &p,
                &before.elapsed(),
            )?;
        }
    }
    if mpi_initialized() {
        unsafe { mpi::ffi::MPI_Finalize() };
    }
    Ok(())
}
