#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

if [[ $TRAVIS == true ]]; then
    TARGET=$HOME/kcov
    export PATH=$TARGET/bin:$PATH
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

files=(target/debug/blkar)

#for file in target/debug/blkar-*[^\.d]; do
# for file in ${files[@]}; do
for file in target/debug/blkar_lib-*; do if [[ $file == *.d ]]; then continue; fi
  # mkdir -p "target/cov/$(basename $file)"
  mkdir -p "target/cov/blkar"
  kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/blkar" "$file"
done
