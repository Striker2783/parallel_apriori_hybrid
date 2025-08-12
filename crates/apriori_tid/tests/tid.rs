use std::{collections::HashSet, path::Path};

use apriori::start::FrequentWriter;
use apriori_tid::{hybrid::AprioriHybridRunner, tid::AprioriTIDRunner2};
use tester::test_utils::{Solved, test_generic};

#[test]
fn test_tid() {
    test_generic("../../test_files", |t, s| {
        let tid = AprioriTIDRunner2::new(&t, s);
        let mut writer: FrequentWriter<HashSet<Vec<usize>>> = FrequentWriter::new();
        tid.run(&mut writer);
        Solved::new(writer.into_inner())
    });
}
#[test]
fn test_hybrid() {
    test_generic(Path::new("../../test_files"), |mut t, s| {
        let tid = AprioriHybridRunner::new(&mut t, s);
        let mut writer: FrequentWriter<HashSet<Vec<usize>>> = FrequentWriter::new();
        tid.run(&mut writer);
        Solved::new(writer.into_inner())
    });
}
