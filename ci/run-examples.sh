#!/bin/sh

examples=$(ls examples | sed "s/\\.rs\$//")

for example in $examples; do
    echo "Running example: $example"
    cargo mpirun --release --example $example
    echo ""
done
