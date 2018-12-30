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

tests=(
  "version_tests"
  "version_tests_encode_stdin"
  "version_tests_decode_stdout"
  "compare_encode_file_and_stdin"
  "compare_decode_file_and_stdout"
  "compare_decode_file_and_stdout_corrupted_container"
  "decode_manual_burst"
  "decode_manual_burst_encode_stdin"
  "decode_manual_burst_decode_stdout"
  "repair_manual_burst"
  "repair_manual_burst_encode_stdin"
  "repair_manual_burst_decode_stdout"
  "nometa_tests"
  "nometa_tests_encode_stdin"
  "nometa_tests_decode_stdout"
  "compare_decode_file_and_stdout_nometa"
  "hash_tests"
  "hash_tests_encode_stdin"
  "hash_tests_decode_stdout"
  "rescue_tests"
  "rescue_tests_encode_stdin"
  "rescue_tests_decode_stdout"
  "rescue_from_to_tests"
  "rescue_from_to_tests_encode_stdin"
  "rescue_pick_uid_tests"
  "rescue_pick_uid_tests_encode_stdin"
  "rescue_pick_uid_tests_decode_stdout"
  "show_from_to_tests"
  "show_pick_uid_tests"
  "out_file_logic_tests"
  "corruption_tests"
  "corruption_tests_encode_stdin"
  "corruption_tests_decode_stdout"
  "burst_corruption_tests"
  "burst_corruption_tests_encode_stdin"
  "burst_corruption_tests_decode_stdout"
  "sort_tests"
  "sort_tests_encode_stdin"
  "sort_tests_decode_stdout"
  "sort_stats_tests"
  "sort_dry_run"
  "sort_multi_pass"
  "decode_multi_pass"
  "file_size_calc_tests"
  "repair_truncated_tests"
  "repair_truncated_tests_encode_stdin"
  "repair_truncated_tests_decode_stdout"
)

test_count=${#tests[@]}

simul_test_count=5

start_date=$(date "+%Y-%m-%d %H:%M")
start_time=$(date "+%s")

echo ""
echo "Test start :" $start_date
echo ""

i=0
while (( $i < $test_count )); do
  if (( $test_count - $i >= $simul_test_count )); then
    tests_to_run=$simul_test_count
  else
    tests_to_run=$[test_count - i]
  fi

  echo "Running $tests_to_run tests in parallel"

  for (( c=0; c < $tests_to_run; c++ )); do
    t=${tests[$i]}
    echo "    Starting $t"

    rm -rf $t/
    mkdir $t/
    cd $t
    ./../gen_dummy.sh
    ./../$t.sh > log 2> stderr_log &
    cd ..

    i=$[i+1]
  done

  echo "Waiting for tests to finish"
  echo ""

  wait
done

# go through all exit codes
test_fail_count=0
tests_failed=()

for t in ${tests[@]}; do
  t_exit_code=$(cat $t/exit_code)

  if (( $t_exit_code != 0 )); then
    echo "========================================"
    echo "Log of $t :"
    echo ""
    cat $t/log
    echo ""
    echo "Stderr log of $t :"
    cat $t/stderr_log
  fi

  if (( $t_exit_code != 0 )); then
    test_fail_count=$[$test_fail_count + 1]
    tests_failed+=("$t")
  fi
done
echo "========================================"

if [[ $test_fail_count == 0 ]]; then
    echo "All tests passed"
    exit_code=0
else
  echo
    echo "$test_fail_count tests failed"
    echo ""
    echo "List of tests failed :"
    for t in ${tests_failed[@]}; do
      echo "    "$t
    done
    exit_code=1
fi

end_date=$(date "+%Y-%m-%d %H:%M")
end_time=$(date "+%s")
echo ""
echo "Test end :" $end_date

echo "Time elapsed :" $[(end_time - start_time) / 60] "minutes"

exit $exit_code
