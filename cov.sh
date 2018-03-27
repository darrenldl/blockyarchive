#!/bin/bash

export RUSTFLAGS="-C link-dead-code"

cargo build

cargo build --tests

files=(target/debug/rsbx)

#for file in target/debug/rsbx-*[^\.d]; do
# for file in ${files[@]}; do
for file in target/debug/rsbx-*; do
  # mkdir -p "target/cov/$(basename $file)"
  mkdir -p "target/cov/rsbx"
  kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/rsbx" "$file"
done

for file in ${files[@]}; do
  # mkdir -p "target/cov/$(basename $file)"
  mkdir -p "target/cov/rsbx"
  kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/rsbx" "$file"
done
