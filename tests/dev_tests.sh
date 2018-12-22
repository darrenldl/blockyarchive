#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

if [[ $PWD != */tests ]]; then
  cd tests
fi

./copy_release.sh

test_failed=0
test_failed_names=""

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
    test_failed_names=$test_failed_names"- version_tests.sh\n"
fi

echo "========================================"
echo "Starting version tests (stdin as encode input)"
echo "========================================"
./version_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- version_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting version tests (stdout as decode output)"
echo "========================================"
./version_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- version_tests_decode_stdout.sh\n"
fi

# compare encode file and stdin input
echo "========================================"
echo "Starting comparison between encode with file input and stdin input"
echo "========================================"
./compare_encode_file_and_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- compare_encode_file_and_stdin.sh\n"
fi

# compare decode file and stdout output
echo "========================================"
echo "Starting comparison between decode with file output and stdout output"
echo "========================================"
./compare_decode_file_and_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- compare_decode_file_and_stdout.sh\n"
fi

echo "========================================"
echo "Starting comparison between decode with file output and stdout output with corrupted container"
echo "========================================"
./compare_decode_file_and_stdout_corrupted_container.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- compare_decode_file_and_stdout_corrupted_container.sh\n"
fi

# decode manual burst tests
echo "========================================"
echo "Starting decode manual burst tests"
echo "========================================"
./decode_manual_burst.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- decode_manual_burst.sh\n"
fi

echo "========================================"
echo "Starting decode manual burst tests (stdin as encode input)"
echo "========================================"
./decode_manual_burst_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- decode_manual_burst_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting decode manual burst tests (stdout as decode output)"
echo "========================================"
./decode_manual_burst_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- decode_manual_burst_decode_stdout.sh\n"
fi

# repair manual burst tests
echo "========================================"
echo "Starting repair manual burst tests"
echo "========================================"
./repair_manual_burst.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- repair_manual_burst.sh\n"
fi

echo "========================================"
echo "Starting repair manual burst tests (stdin as encode input)"
echo "========================================"
./repair_manual_burst_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- repair_manual_burst_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting repair manual burst tests (stdout as decode output)"
echo "========================================"
./repair_manual_burst_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- repair_manual_burst_decode_stdout.sh\n"
fi

# nometa tests
echo "========================================"
echo "Starting nometa tests"
echo "========================================"
./nometa_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- nometa_tests.sh\n"
fi

echo "========================================"
echo "Starting nometa tests (stdin as encode input)"
echo "========================================"
./nometa_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- nometa_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting nometa tests (stdout as decode output)"
echo "========================================"
./nometa_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- nometa_tests_decode_stdout.sh\n"
fi

echo "========================================"
echo "Starting comparison between decode with file output and stdout output with nometa"
echo "========================================"
./compare_decode_file_and_stdout_nometa.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- compare_decode_file_and_stdout_nometa.sh\n"
fi

# hash test
echo "========================================"
echo "Starting hash tests"
echo "========================================"
./hash_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- hash_tests.sh\n"
fi

echo "========================================"
echo "Starting hash tests (stdin as encode input)"
echo "========================================"
./hash_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- hash_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting hash tests (stdout as decode output)"
echo "========================================"
./hash_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- hash_tests_decode_stdout.sh\n"
fi

# rescue tests
echo "========================================"
echo "Starting rescue tests"
echo "========================================"
./rescue_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- rescue_tests.sh\n"
fi

echo "========================================"
echo "Starting rescue tests (stdin as encode input)"
echo "========================================"
./rescue_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- rescue_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting rescue tests (stdout as decode output)"
echo "========================================"
./rescue_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- rescue_tests_decode_stdout.sh\n"
fi

echo "========================================"
echo "Starting rescue with specified range tests"
echo "========================================"
./rescue_from_to_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- rescue_from_to_tests.sh\n"
fi

echo "========================================"
echo "Starting rescue with specified range tests (stdin as encode input)"
echo "========================================"
./rescue_from_to_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- rescue_from_to_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting rescue with specified range tests (stdout as decode output)"
echo "========================================"
echo "Non applicable, skipped"

echo "========================================"
echo "Starting rescue with specified uid tests"
echo "========================================"
./rescue_pick_uid_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- rescue_pick_uid_tests.sh\n"
fi

echo "========================================"
echo "Starting rescue with specified uid tests (stdin as encode input)"
echo "========================================"
./rescue_pick_uid_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- rescue_pick_uid_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting rescue with specified uid tests (stdout as decode output)"
echo "========================================"
./rescue_pick_uid_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- rescue_pick_uid_tests_decode_stdout.sh\n"
fi

# output file tests
echo "========================================"
echo "Starting output file path logic tests"
echo "========================================"
./out_file_logic_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- out_file_logic_tests.sh\n"
fi

# corruption tests
echo "========================================"
echo "Starting corruption tests"
echo "========================================"
./corruption_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- corruption_tests.sh\n"
fi

echo "========================================"
echo "Starting corruption tests (stdin as encode input)"
echo "========================================"
./corruption_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- corruption_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting corruption tests (stdout as decode output)"
echo "========================================"
./corruption_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- corruption_tests_decode_stdout.sh\n"
fi

# burst corruption tests
echo "========================================"
echo "Starting burst corruption tests"
echo "========================================"
./burst_corruption_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- burst_corruption_tests.sh\n"
fi

echo "========================================"
echo "Starting burst corruption tests (stdin as encode input)"
echo "========================================"
./burst_corruption_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- burst_corruption_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting burst corruption tests (stdout as decode output)"
echo "========================================"
./burst_corruption_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- burst_corruption_tests_decode_stdout.sh\n"
fi

# sort tests
echo "========================================"
echo "Starting sort tests"
echo "========================================"
./sort_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- sort_tests.sh\n"
fi

echo "========================================"
echo "Starting sort tests (stdin as encode input)"
echo "========================================"
./sort_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- sort_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting sort tests (stdout as decode output)"
echo "========================================"
./sort_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- sort_tests_decode_stdout.sh\n"
fi

# file size tests
echo "========================================"
echo "Starting file size calculation tests"
echo "========================================"
./file_size_calc_tests.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- file_size_calc_tests.sh\n"
fi

# container truncation tests
echo "========================================"
echo "Starting truncated container repair tests"
echo "========================================"
./repair_truncated_tests.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- repair_truncated_tests.sh\n"
fi

echo "========================================"
echo "Starting truncated container repair tests (stdin as encode input)"
echo "========================================"
./repair_truncated_tests_encode_stdin.sh
if [[ $? != 0 ]]; then
  test_failed=$[$test_failed+1]
  test_failed_names=$test_failed_names"- repair_truncated_tests_encode_stdin.sh\n"
fi

echo "========================================"
echo "Starting truncated container repair tests (stdout as decode output)"
echo "========================================"
./repair_truncated_tests_decode_stdout.sh
if [[ $? != 0 ]]; then
    test_failed=$[$test_failed+1]
    test_failed_names=$test_failed_names"- repair_truncated_tests_decode_stdout.sh\n"
fi

if [[ $test_failed == 0 ]]; then
    echo "All tests passed"
    exit 0
else
    echo "$test_failed tests failed"
    echo "List of tests failed"
    echo -e $test_failed_names
    exit 1
fi
