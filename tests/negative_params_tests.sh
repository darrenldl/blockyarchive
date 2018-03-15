#!/bin/bash

truncate -s 1M dummy

./rsbx encode dummy -f &> /dev/null

VALS=(1 0 -1)

for v in ${VALS[*]}; do
  echo "show using $v skip-to"
  echo "========================================"

  ./rsbx show --skip-to="$v" dummy.sbx

  echo ""
done

for v in ${VALS[*]}; do
    echo "show using $v to"
    echo "========================================"

    ./rsbx show --to="$v" dummy.sbx

    echo ""
done
