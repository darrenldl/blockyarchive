#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

# export RUSTFLAGS="-C link-dead-code"

cargo build
if [[ $? != 0 ]]; then
  exit 1
fi

cargo build --tests
if [[ $? != 0 ]]; then
  exit 1
fi

files=(target/debug/rsbx)

#for file in target/debug/rsbx-*[^\.d]; do
# for file in ${files[@]}; do
for file in target/debug/rsbx-*; do if [[ $file == *.d ]]; then continue; fi
  # mkdir -p "target/cov/$(basename $file)"
  mkdir -p "target/cov/rsbx"
  kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/rsbx" "$file"
done
