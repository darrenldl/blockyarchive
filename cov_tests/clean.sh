#!/bin/bash

if [[ $PWD != */cov_tests ]]; then
  cd cov_tests
fi

rm dummy* &>/dev/null

rm *.sbx &>/dev/null

rm rescued_data/* &>/dev/null

rm rescued_data2/* &>/dev/null

rm rescue_log &>/dev/null

rm filler* &>/dev/null

rm out_test/* &>/dev/null
