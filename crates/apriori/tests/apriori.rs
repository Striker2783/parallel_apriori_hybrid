use std::collections::HashSet;

use apriori::{
    alone::AprioriTrie,
    apriori::AprioriRunner,
    start::{Apriori, FrequentWriter},
};
use tester::test_utils::{Solved, test_generic};

#[test]
fn test_apriori() {
    test_generic("../../test_files", |t, s| {
        let a = AprioriRunner::new(&t, s);
        let mut writer: FrequentWriter<HashSet<Vec<usize>>> = FrequentWriter::new();
        a.run(&mut writer);
        Solved::new(writer.into_inner())
    });
}
#[test]
fn test_apriori_trie() {
    test_generic("../../test_files", |t, s| {
        let a = AprioriTrie::new(t, s);
        let mut writer: FrequentWriter<HashSet<Vec<usize>>> = FrequentWriter::new();
        a.run(&mut writer);
        Solved::new(writer.into_inner())
    });
}
