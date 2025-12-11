# Parallelization of AprioriHybrid

Paper: [Parallelization of AprioriHybrid](./paper.pdf)

ICoDSE Conference: https://icodse.org/

Publication Pending

## Repository Description

This repository contains the source code for the implementation of this paper. The implementation was written in Rust with `rsmpi` for MPI support. It also contains a `singularity.def` file to build a container to run on a HPC cluster. It also contains a Github workflow that automates the testing and building of the implementation.

## Build Instructions

1. Install Rust from https://rust-lang.org/
2. Install OpenMPI from https://www.open-mpi.org/
3. Clone this repository.

```
git clone https://github.com/Striker2783/parallel_apriori_hybrid.git
```

4. Navigate to the project directory.

```
cd parallel_apriori_hybrid
```

5. Build using Cargo

```
cargo build --release
```

6. Run the program

```
cargo run --release --
```

or

```
./target/release/parallel_apriori
```

## Using Singularity

1. Install SingularityCE from https://sylabs.io/singularity/
2. Clone this repository.

```
git clone https://github.com/Striker2783/parallel_apriori_hybrid.git
```

3. Navigate to the project directory.

```
cd parallel_apriori_hybrid
```

4. Build the container

```
singularity build singularity.def image.sif
```

5. Run the container

```
singularity exec image.sif /p
```

## Run with OpenMPI

```
mpirun ./target/release/parallel_apriori
```

## CLI Options

- File: the input file to the algorithm. The file should be a list of transactions separated by lines. Each transaction is a list of item IDs separated by spaces. Some examples are in https://fimi.uantwerpen.be/data/ .

- Support Count: the minimum support count for an item set to be considered frequent.

- Algorithm: the frequent itemset mining algorithm to be used.

  - apriori: the Apriori algorithm
  - apriori-tid: the AprioriTID algorithm
  - apriori-hybrid: the AprioriHybrid algorithm
  - apriori-trie: Apriori with a trie
  - count-distribution-hybrid: the parallelized AprioriHybrid algorithm
  - count-distribution: the parallelized Apriori algorithm

- Output file `-o <File>`. This is where all the frequent itemsets are stored to. It generates a list of frequent itemsets separated by spaces.

- CSV file `-csv <File>`. This appends the support percentage and time taken to a CSV file.

- Profiler file `--profiler <File>`. This will generate a profile file that can be used with `pprof`.

- Time `-t`. This option prints out the running time of the algorithm to standard out.
