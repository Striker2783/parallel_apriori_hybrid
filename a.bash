#!/bin/bash

# Check if at least one argument was provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 {test|profile}"
    exit 1
fi
N=2500
case "$1" in
    test)
        while [ $N -gt 700 ]
        do
            # mpirun ./target/release/parallel_apriori files/T40.dat $N count-distribution-hybrid
            cargo run --release -- files/T40.dat $N apriori-hybrid
            N=$((N - 100))
        done
        ;;
    profile)
        cargo build --release
        shift
        samply record ./target/release/parallel_apriori $@
        ;;
    profilep)
        cargo build --release
        shift
        samply record mpirun ./target/release/parallel_apriori $@
        ;;
    parallel)
        cargo build --release
        shift
        mpirun ./target/release/parallel_apriori $@
        ;;
    *)
        echo "Invalid option: $1"
        echo "Usage: $0 {test|profile}"
        exit 1
        ;;
esac