#!/bin/bash

source kcov_blkar_fun.sh

exit_code=0

echo "Generating test data"
truncate -s 1K dummy

mkdir out_test &>/dev/null
rm out_test/* &>/dev/null

echo -n "Testing encode output with no provided path"
rm dummy.sbx &>/dev/null
kcov_blkar encode dummy &>/dev/null

if [ -f "dummy.sbx" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Testing encode output with provided full file path"
kcov_blkar encode dummy out_test/dummy1.sbx &>/dev/null

if [ -f "out_test/dummy1.sbx" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Testing encode output with provided directory path"
kcov_blkar encode dummy out_test &>/dev/null

if [ -f "out_test/dummy.sbx" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Testing decode output with no provided path"
rm dummy &>/dev/null
kcov_blkar decode dummy.sbx &>/dev/null

if [ -f "dummy" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Testing decode output with provided full file path"
kcov_blkar decode dummy.sbx out_test/decoded &>/dev/null

if [ -f "out_test/decoded" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Testing decode output with provided directory path"
kcov_blkar decode dummy.sbx out_test &>/dev/null

if [ -f "out_test/dummy" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Regenrating test data"
truncate -s 1K dummy

echo "Encode with no metadata"
rm dummy.sbx &>/dev/null
kcov_blkar encode --sbx-version 1 dummy --no-meta &>/dev/null

echo "Repeating same tests for decoding"
echo -n "Testing decode output with no provided path"
rm dummy &>/dev/null
kcov_blkar decode dummy.sbx &>/dev/null

if [ ! -f "dummy" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

rm out_test/* &>/dev/null

echo -n "Testing decode output with provided full file path"
kcov_blkar decode dummy.sbx out_test/decoded &>/dev/null

if [ -f "out_test/decoded" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Testing decode output with provided directory path"
kcov_blkar decode dummy.sbx out_test &>/dev/null

if [ ! -f "out_test/dummy" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

exit $exit_code
