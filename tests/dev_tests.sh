#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

if [[ $PWD != */tests ]]; then
  cd tests
fi

if [[ "$1" == "debug" ]]; then
  ./copy.sh
else
  ./copy_release.sh
fi

test_failed=0

source test_list.sh

test_count=${#tests[@]}

simul_test_count=5

start_date=$(date "+%Y-%m-%d %H:%M")
start_time=$(date "+%s")

tests_missing=0
for t in ${tests[@]}; do
    if [ ! -f $t.sh ]; then
        echo "Test $t.sh is missing"
    fi
    tests_missing=$[tests_missing + 1]
done

if [[ $tests_missing != 0 ]]; then
    exit 1
fi

echo ""
echo "Test start :" $start_date
echo ""

echo "Starting $test_count tests"
echo ""

i=0
while (( $i < $test_count )); do
  if (( $test_count - $i >= $simul_test_count )); then
    tests_to_run=$simul_test_count
  else
    tests_to_run=$[test_count - i]
  fi

  echo "Running $tests_to_run tests in parallel"

  j=$i

  for (( c=0; c < $tests_to_run; c++ )); do
    t=${tests[$i]}
    if [[ "$t" != "" ]]; then
      echo "    Starting $t"

      rm -rf $t/
      mkdir $t/
      cd $t
      ./../gen_dummy.sh
      cp ../functions.sh .
      ./../$t.sh > log 2> stderr_log &
      cd ..

      i=$[i+1]
    fi
  done

  echo "Waiting for tests to finish"

  wait

  echo "Cleaning up files"

  for (( c=0; c < $tests_to_run; c++ )); do
    t=${tests[$j]}

    if [[ "$t" != "" ]]; then
      cd $t

      if [[ $? == 0 ]]; then
        find . -type f \
             -not -name "exit_code" \
             -not -name "log" \
             -not -name "stderr_log" \
             -delete

        cd ..
      fi
    fi

    j=$[j+1]
  done

  echo ""
  echo "$[test_count - i] / $test_count tests remaining"
  echo ""
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
    echo "All $test_count tests passed"
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
