#!/bin/bash

TARGET=$HOME/kcov

export PATH=$TARGET/bin:$PATH

function kcov_rsbx() {
    if [[ $TRAVIS == true ]]; then

    else
        kcov --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@"
    fi
}
