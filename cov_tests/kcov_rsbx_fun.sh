#!/bin/bash

TARGET=$HOME/kcov

if [[ $TRAVIS == true ]]; then
    export PATH=$TARGET/bin:$PATH
fi

mkdir -p "$HOME/target/cov/rsbx"

function kcov_rsbx() {
    kcov --exclude-pattern=/.cargo,/usr/lib --verify "$HOME/target/cov/rsbx" rsbx "$@"; bash <(curl -s https://codecov.io/bash)
    kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify "$HOME/target/cov/rsbx" rsbx "$@"
}
