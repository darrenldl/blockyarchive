#!/bin/bash

cd ..

echo "Building osbx"
jbuilder build @install
echo ""

echo "Copying osbx binary over"
cp _build/default/src/osbx.exe ./tests/osbx
echo ""

cd tests

echo "Generating test data"
dd if=/dev/urandom of=dummy bs=$[1024 * 1024 * 1] count=1
# truncate -s 10m dummy
echo ""

# version tests
echo "Starting version tests"
echo "========================================"
./version_tests.sh
echo "========================================"

echo ""

# nometa tests
echo "Starting nometa tests"
echo "========================================"
./nometa_tests.sh
echo "========================================"

echo ""

# hash test
echo "Starting hash tests"
echo "========================================"
./hash_tests.sh
echo "========================================"

echo ""

# rescue tests
echo "Starting rescue tests"
echo "========================================"
./rescue_tests.sh
echo "========================================"

echo ""

echo "Starting rescue with specified range tests"
echo "========================================"
./rescue_from_to_tests.sh
echo "========================================"

echo ""

echo "Starting rescue with specified uid tests"
echo "========================================"
./rescue_pick_uid_tests.sh
echo "========================================"

echo ""

# output file tests
echo "Starting output file path logic tests"
echo "========================================"
./out_file_logic_tests.sh
echo "========================================"
