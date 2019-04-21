#!/bin/bash

if [[ $PWD != */blockyarchive/tests ]]; then
  echo "Please invoke clean.sh in the tests directory"
  exit 1
fi

source test_list.sh

rm -f dummy*

rm -f  *.sbx

rm -f  *.ecsbx

rm -f rescued_data/*

rm -f rescued_data2/*

rm -f rescue_log

rm -f filler*

rm -f out_test/*

rm -f sort_*.sbx.*

rm -f sort_*.ecsbx.*

rm -f exit_code

rm -f ../blkar

rm -f data_chunk

rm -f data_chunk_orig

rm -f chunk_*

rm -f decode*.sbx.*

rm -f decode*.ecsbx.*

find . -regextype sed -regex "./sort_[0-9]*_[0-9]*_[0-9]*" -delete

find . -regextype sed -regex "./decode_[0-9]*_[0-9]*_[0-9]*" -delete

for t in ${tests[@]}; do
  if [[ "$t" != "" ]]; then
    rm -rf $t
  fi
done
