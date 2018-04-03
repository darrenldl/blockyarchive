#!/bin/bash

if ! [[ $TARGET == x86_64-unknown-linux-gnu && DISABLE_COV == 0 ]]; then
    exit 0
fi

cd tests

./copy_release.sh

test_failed=0

echo "Generating test data"
dd if=/dev/urandom of=dummy bs=$[1024 * 1024 * 1] count=1 &>/dev/null
# truncate -s 10m dummy

# version tests
echo "========================================"
echo "Starting version tests"
echo "========================================"
./version_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# nometa tests
echo "========================================"
echo "Starting nometa tests"
echo "========================================"
./nometa_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# hash test
echo "========================================"
echo "Starting hash tests"
echo "========================================"
./hash_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# rescue tests
echo "========================================"
echo "Starting rescue tests"
echo "========================================"
./rescue_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting rescue with specified range tests"
echo "========================================"
./rescue_from_to_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting rescue with specified uid tests"
echo "========================================"
./rescue_pick_uid_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# output file tests
echo "========================================"
echo "Starting output file path logic tests"
echo "========================================"
./out_file_logic_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# corruption tests
echo "========================================"
echo "Starting corruption tests"
echo "========================================"
./corruption_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# burst corruption tests
echo "========================================"
echo "Starting burst corruption tests"
echo "========================================"
./burst_corruption_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# sort tests
echo "========================================"
echo "Starting sort tests"
echo "========================================"
./sort_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# file size tests
echo "========================================"
echo "Starting file size calculation tests"
echo "========================================"
./file_size_calc_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

# container truncation tests
echo "========================================"
echo "Starting truncated container repair tests"
echo "========================================"
./repair_truncated_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

if [[ $test_failed == 0 ]]; then
    echo "All tests passed"
    exit 0
else
    echo "$test_failed tests failed"
    exit 1
fi
