#!/bin/bash

cd ..

echo "Building rsbx"
cargo build

echo "Copying rsbx binary over"
cp target/debug/rsbx ./cov_tests/rsbx
cd cov_tests
