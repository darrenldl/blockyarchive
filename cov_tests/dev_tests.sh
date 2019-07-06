#!/bin/bash

if [[ $TRAVIS == true ]]; then
    if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
        exit 0
    fi
fi

if [[ $PWD != */cov_tests ]]; then
    cd cov_tests
fi

./copy.sh

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
        tests_missing=$[tests_missing + 1]
    fi
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
      cp ../kcov_blkar_fun.sh .
      (echo $(date "+%s") > start_time; ./../$t.sh > log 2> stderr_log; echo $(date "+%s") > end_time) &
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
             -not -name "dummy_file_size" \
             -not -name "start_time" \
             -not -name "end_time" \
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

echo "========================================"
echo ""

echo "Merging coverage reports"
# merge coverage support
cov_dirs=""
for t in ${tests[@]}; do
    cov_dirs=$cov_dirs" "$t/cov
done
merged_cov_dir="../target/cov/bin-tests"
rm -rf $merged_cov_dir
mkdir -p $merged_cov_dir
kcov --merge $merged_cov_dir $cov_dirs

echo ""

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

echo ""

echo "Time elapsed :" $[(end_time - start_time) / 60] "minutes"

if [[ "$1" == "reorder" || "$2" == "reorder" ]]; then
    echo ""

    echo "Reordering test list by time taken"

    rm -f test_stats
    for t in ${tests[@]}; do
        start_time=$(cat $t/start_time)
        end_time=$(cat $t/end_time)
        time_elapsed=$[(end_time - start_time) / 10 * 10]

        echo "$t $time_elapsed" >> test_stats
    done

    sort test_stats -k2n > test_stats_sorted

    echo '#!/bin/bash' > test_list.sh
    echo "" >> test_list.sh
    echo "tests=(" >> test_list.sh
    while IFS= read -r line; do
        test=$(echo $line | awk '{ print $1 }')
        echo "    \"$test\"" >> test_list.sh
    done < test_stats_sorted
    echo ")" >> test_list.sh
fi

exit $exit_code
