use std::collections::HashSet;

use apriori::start::FrequentWriter;
use count_distribution::hybridrunner::CountDistributionHybrid;
use mpi::traits::Communicator;
use parallel::traits::ParallelRun;
use tester::test_utils::test_generic_with_paths;

fn main() {
    let universe = mpi::initialize();
    let universe = universe.unwrap();
    let world = universe.world();
    assert!(world.size() > 1);
    test_generic_with_paths("./test_files", |t, s| {
        let mut writer: FrequentWriter<HashSet<Vec<usize>>> = FrequentWriter::new();
        let a = CountDistributionHybrid::new(&t, s, &mut writer);
        a.run(&universe);
        if world.rank() == 0 {
            Some(writer.into_inner().into())
        } else {
            None
        }
    });
}
