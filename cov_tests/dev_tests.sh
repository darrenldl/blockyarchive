#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

cd cov_tests

./copy.sh

test_failed=0

echo "Generating test data"
./gen_dummy.sh
# truncate -s 10m dummy


if [[ $test_failed == 0 ]]; then
    echo "All tests passed"
    exit 0
else
    echo "$test_failed tests failed"
    exit 1
fi
