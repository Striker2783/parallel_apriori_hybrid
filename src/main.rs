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
use pprof::ProfilerGuard;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write as IOWrite};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
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
    #[arg(long)]
    profiler: Option<PathBuf>,
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
    guard: bool,
}

impl<T: Write> Inputs<T> {
    pub fn new(data: PathBuf, support_count: u64, out: T, guard: bool) -> Self {
        Self {
            data,
            support_count,
            out,
            guard,
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
static GUARD: OnceLock<ProfilerGuard> = OnceLock::new();

pub fn get_guard() -> &'static ProfilerGuard<'static> {
    GUARD.get_or_init(|| ProfilerGuard::new(100).unwrap())
}

pub fn guard_initialized() -> bool {
    GUARD.get().is_some()
}

pub fn get_universe() -> &'static Universe {
    MPI_UNIVERSE.get_or_init(|| environment::initialize().expect("Failed to initialize MPI"))
}

pub fn mpi_initialized() -> bool {
    MPI_UNIVERSE.get().is_some()
}

fn aa<T: Write>(mut input: Inputs<T>, v: &Args) -> Result<(), std::io::Error> {
    match v.algorithm {
        Algorithms::Apriori => {
            if input.guard {
                get_guard();
            }
            let data = TransactionSet::from_path(&input.data)?;
            let runner = AprioriRunner::new(Arc::new(data), input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::CountDistribution => {
            let universe = get_universe();
            if input.guard {
                get_guard();
            }
            let runner = CountDistribution::new(&input.data, input.support_count, &mut input.out);
            runner.run(universe);
        }
        Algorithms::AprioriTID => {
            if input.guard {
                get_guard();
            }
            let data = TransactionSet::from_path(&input.data)?;
            let runner = AprioriTIDRunner2::new(&data, input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::AprioriHybrid => {
            if input.guard {
                get_guard();
            }
            let mut data = TransactionSet::from_path(&input.data)?;
            let runner = AprioriHybridRunner::new(&mut data, input.support_count);
            runner.run(&mut input.out);
        }
        Algorithms::CountDistributionHybrid => {
            let universe = get_universe();
            if input.guard {
                get_guard();
            }
            let runner =
                CountDistributionHybrid::new(&input.data, input.support_count, &mut input.out);
            runner.run(universe);
        }
        Algorithms::AprioriTrie => {
            if input.guard {
                get_guard();
            }
            let runner =
                AprioriTrie::new(TransactionSet::from_path(&input.data)?, input.support_count);
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
            let input = Inputs::new(
                a.file.clone(),
                a.support_count,
                writer,
                a.profiler.is_some(),
            );
            aa(input, &a).unwrap();
        }
        None => {
            let input = Inputs::new(
                a.file.clone(),
                a.support_count,
                EmptyWriter::new(),
                a.profiler.is_some(),
            );
            aa(input, &a).unwrap();
        }
    };
    if a.time {
        println!("Time Taken: {:?}", before.elapsed());
    }
    if let Some(p) = a.csv {
        if !mpi_initialized() || (mpi_initialized() && get_universe().world().rank() == 0) {
            output_csv(&p, &before.elapsed())?;
        }
    }
    if let Some(mut flamegraph) = a.profiler {
        if guard_initialized() {
            if let Ok(report) = get_guard().report().build() {
                if mpi_initialized() {
                    let mut stem = flamegraph.file_stem().unwrap().to_os_string();
                    stem.push(get_universe().world().rank().to_string());
                    stem.push(".");
                    stem.push(flamegraph.extension().unwrap());
                    flamegraph.set_file_name(stem);
                }
                let file = std::fs::File::create(&flamegraph).unwrap();
                report.flamegraph(file).unwrap();
            }
        } else {
            println!("Guard did not intialize");
        }
    }
    if mpi_initialized() {
        unsafe { mpi::ffi::MPI_Finalize() };
    }
    Ok(())
}
