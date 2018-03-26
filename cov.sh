#!/bin/bash

cargo build

cargo test

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
