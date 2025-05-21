#!/bin/bash

# Check if at least one argument was provided
if [ $# -eq 0 ]; then
    echo "Usage: $0 {test|profile}"
    exit 1
fi

case "$1" in
    test)
        echo "Running in test mode"
        # Add your test commands here
        ;;
    profile)
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