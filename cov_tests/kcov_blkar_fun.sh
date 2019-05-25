#!/bin/bash

COV_DIR=cov

mkdir -p $COV_DIR

if [[ $TRAVIS == true ]]; then
    TARGET=$HOME/kcov
    export PATH=$TARGET/bin:$PATH
fi

blkar() {
    if [[ $TRAVIS == true ]]; then
        kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify $COV_DIR ../blkar "$@" | sed "s/kcov.*//"
    else
        kcov --exclude-pattern=/.cargo,/usr/lib --verify $COV_DIR ../blkar "$@" | sed "s/kcov.*//"
    fi
}
