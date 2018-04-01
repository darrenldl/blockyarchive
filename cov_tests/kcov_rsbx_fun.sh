#!/bin/bash

TARGET=$HOME/kcov

if [[ $TRAVIS == true ]]; then
    export PATH=$TARGET/bin:$PATH
fi

function kcov_rsbx() {
    mkdir -p "../target/cov/rsbx"
    if [[ $TRAVIS == true ]]; then
        kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@" &>/dev/null
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@"
        #./rsbx "$@"
    else
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@"
    fi
}
