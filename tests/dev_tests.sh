#!/bin/bash

cd tests

./copy.sh

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

if [[ $test_failed == 0 ]]; then
    echo "All tests passed"
    exit 0
else
    echo "$test_failed tests failed"
    exit 1
fi
