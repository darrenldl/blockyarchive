#!/bin/bash

cd ..

echo "Building blkar"
cargo build

echo "Copying blkar binary over"
cp target/debug/blkar ./tests/blkar
cp target/debug/blkar ./blkar

