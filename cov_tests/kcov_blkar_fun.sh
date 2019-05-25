#!/bin/bash

mkdir -p "../../target/cov/blkar"

TARGET=$HOME/kcov

if [[ $TRAVIS == true ]]; then
    export PATH=$TARGET/bin:$PATH
fi

blkar() {
    if [[ $TRAVIS == true ]]; then
        kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify "../../target/cov/blkar" blkar "$@" | sed "s/kcov.*//"
    else
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "../../target/cov/blkar" ./blkar "$@" | sed "s/kcov.*//"
    fi
}
