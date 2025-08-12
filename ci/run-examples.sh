#!/bin/sh

echo "IDK WHY PERMISSION DENIED"

examples=$(find examples -maxdepth 1 -type f -name '*.rs' -exec basename {} .rs \;)

for example in $examples; do
    echo "Running example: $example"
    cargo mpirun --release --example $example
    echo ""
done
