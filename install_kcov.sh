#!/bin/bash

if ! [[ $TARGET == x86_64-unknown-linux-gnu && DISABLE_COV == 0 ]]; then
    exit 0
fi

set -e
shopt -s nullglob

TARGET=$HOME/kcov

wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
cd kcov-master
mkdir build
cd build
cmake .. -DCMAKE_INSTALL_PREFIX=$TARGET
make
make install
cd ../..
