#!/bin/bash

TARGET=$HOME/kcov

if [[ $TRAVIS == true ]]; then
    export PATH=$TARGET/bin:$PATH
fi

mkdir -p "$HOME/target/cov/rsbx"

function kcov_rsbx() {
    if [[ $TRAVIS == true ]]; then
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "$HOME/target/cov/rsbx" rsbx "$@"
        kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify "$HOME/target/cov/rsbx" rsbx "$@"
        ./rsbx "@"
    else
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@"
    fi
}
