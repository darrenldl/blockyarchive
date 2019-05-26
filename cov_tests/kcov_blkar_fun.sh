#!/bin/bash

COV_DIR=cov

mkdir -p $COV_DIR

if [[ $TRAVIS == true ]]; then
    export PATH=$HOME/kcov/bin:$PATH
fi

blkar() {
    kcov --exclude-pattern=/.cargo,/usr/lib --verify $COV_DIR ../blkar "$@" | sed "s/kcov.*//"
}
