#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

cd tests

./copy_release.sh

test_failed=0

echo "Generating test data"
./gen_dummy.sh
# truncate -s 10m dummy

# version tests
echo "========================================"
echo "Starting version tests"
echo "========================================"
./version_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting version tests (stdin as encode input)"
echo "========================================"
./version_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting version tests (stdout as decode output)"
echo "========================================"
./version_tests_decode_stdout.sh
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

echo "========================================"
echo "Starting nometa tests (stdin as encode input)"
echo "========================================"
./nometa_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting nometa tests (stdout as decode output)"
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

echo "========================================"
echo "Starting hash tests"
echo "========================================"
./hash_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting hash tests"
echo "========================================"
./hash_tests_decode_stdout.sh
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
echo "Starting rescue tests (stdin as encode input)"
echo "========================================"
./rescue_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting rescue tests (stdout as decode output)"
echo "========================================"
./rescue_tests_decode_stdout.sh
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
echo "Starting rescue with specified range tests (stdin as encode input)"
echo "========================================"
./rescue_from_to_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting rescue with specified range tests (stdout as encode output)"
echo "========================================"
echo "Non applicable, skipped"

echo "========================================"
echo "Starting rescue with specified uid tests"
echo "========================================"
./rescue_pick_uid_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting rescue with specified uid tests (stdin as encode input)"
echo "========================================"
./rescue_pick_uid_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting rescue with specified uid tests (stdout as decode output)"
echo "========================================"
./rescue_pick_uid_tests_decode_stdout.sh
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

echo "========================================"
echo "Starting corruption tests (stdin as encode input)"
echo "========================================"
./corruption_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting corruption tests (stdout as decode output)"
echo "========================================"
./corruption_tests_decode_stdout.sh
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

echo "========================================"
echo "Starting burst corruption tests (stdin as encode input)"
echo "========================================"
./burst_corruption_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
fi

echo "========================================"
echo "Starting burst corruption tests (stdout as decode output)"
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
