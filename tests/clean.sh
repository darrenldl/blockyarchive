#!/bin/bash

if [[ $PWD != */tests ]]; then
  cd tests
fi

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
  "check_from_to_tests"
  "decode_from_to_tests_corruption_based"
  "decode_from_to_tests"
  "decode_from_to_tests_decode_stdout"
  "encode_from_to_tests"
)

rm -f dummy*

rm -f  *.sbx

rm -f rescued_data/*

rm -f rescued_data2/*

rm -f rescue_log

rm -f filler*

rm -f out_test/*

rm -f sort_*.sbx.*

rm -f exit_code

rm -f ../blkar

rm -f data_chunk

rm -f data_chunk_orig

exit

for t in ${tests[@]}; do
  if [[ "$t" != "" ]]; then
    rm -rf $t
  fi
done
