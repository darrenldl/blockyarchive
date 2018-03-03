#!/bin/bash

truncate -s 1M dummy

./osbx encode dummy -f &> /dev/null

VALS=(1 0 -1)

for v in ${VALS[*]}; do
  echo "decode using $v show-fail-max"
  echo "========================================"

  ./osbx decode --show-fail-max=$v dummy.sbx -f

  echo ""
done

for v in ${VALS[*]}; do
  echo "show using $v find-max"
  echo "========================================"

  ./osbx show --find-max=$v dummy.sbx

  echo ""
done

for v in ${VALS[*]}; do
  echo "show using $v skip-to"
  echo "========================================"

  ./osbx show --skip-to=$v dummy.sbx

  echo ""
done
