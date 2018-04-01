#!/bin/bash

function kcov_rsbx() {
    kcov --exclude-pattern=/.cargo,/usr/lib --verify "../target/cov/rsbx" rsbx "$@"
}
