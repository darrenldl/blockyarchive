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

cargo build --tests
if [[ $? != 0 ]]; then
    exit 1
fi

files=(target/debug/blkar)

echo "Running cargo tests"
COV_DIR="target/cov/cargo-tests"
for file in target/debug/blkar_lib-*; do
    if [[ $file == *.d ]]; then
        continue
    fi

    mkdir -p $COV_DIR
    kcov --exclude-pattern=/.cargo,/usr/lib --verify $COV_DIR "$file"
done

echo "Running binary tests"
cd cov_tests/
./dev_tests.sh
if [[ $? != 0 ]]; then
    exit 1
fi
cd ..

echo "Merging all code coverage reports"
rm -rf target/cov/total
mkdir -p target/cov/total
kcov --merge target/cov/total $COV_DIR target/cov/bin-tests
