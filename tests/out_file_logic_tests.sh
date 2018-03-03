#!/bin/bash

echo "Generating test data"
truncate -s 1K dummy

echo ""

mkdir out_test
rm out_test/*

echo "Testing encode output with no provided path"
rm dummy.sbx
echo ""
./osbx encode dummy

if [ -f "dummy.sbx" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Testing encode output with provided full file path"
echo ""
./osbx encode dummy out_test/dummy1.sbx

if [ -f "out_test/dummy1.sbx" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Testing encode output with provided directory path"
echo ""
./osbx encode dummy out_test

if [ -f "out_test/dummy.sbx" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Testing decode output with no provided path"
rm dummy
echo ""
./osbx decode dummy.sbx

if [ -f "dummy" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Testing decode output with provided full file path"
echo ""
./osbx decode dummy.sbx out_test/decoded

if [ -f "out_test/decoded" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Testing decode output with provided directory path"
echo ""
./osbx decode dummy.sbx out_test

if [ -f "out_test/dummy" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Regenrating test data"
truncate -s 1K dummy
echo ""

echo "Encode with no metadata"
rm dummy.sbx
echo ""
./osbx encode dummy --no-meta

echo ""

echo "Repeating same tests for decoding"
echo "Testing decode output with no provided path"
rm dummy
echo ""
./osbx decode dummy.sbx

if [ ! -f "dummy" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

rm out_test/*

echo "Testing decode output with provided full file path"
echo ""
./osbx decode dummy.sbx out_test/decoded

if [ -f "out_test/decoded" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi

echo ""

echo "Testing decode output with provided directory path"
echo ""
./osbx decode dummy.sbx out_test

if [ ! -f "out_test/dummy" ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi
