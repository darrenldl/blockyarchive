#!/bin/bash

mkdir -p "../target/cov/rsbx"

TARGET=$HOME/kcov

if [[ $TRAVIS == true ]]; then
    export PATH=$TARGET/bin:$PATH
fi

kcov_rsbx() {
    if [[ $TRAVIS == true ]]; then
        kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@" | sed "s/kcov.*//"
    else
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" ./rsbx "$@" | sed "s/kcov.*//"
    fi
}
