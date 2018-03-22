#!/bin/bash

cd ..

echo "Building rsbx"
cargo build --release

echo "Copying rsbx binary over"
cp target/release/rsbx ./tests/rsbx
cd tests
