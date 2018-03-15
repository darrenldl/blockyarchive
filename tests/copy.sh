#!/bin/bash

cd ..

echo "Building rsbx"
cargo build
echo ""

echo "Copying rsbx binary over"
cp target/debug/rsbx ./tests/rsbx
echo ""

cd tests
