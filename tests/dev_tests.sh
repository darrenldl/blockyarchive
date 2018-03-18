#!/bin/bash

cd tests

./copy.sh

echo "Generating test data"
dd if=/dev/urandom of=dummy bs=$[1024 * 1024 * 1] count=1 &>/dev/null
# truncate -s 10m dummy

# version tests
echo "========================================"
echo "Starting version tests"
echo "========================================"
./version_tests.sh

# nometa tests
echo "========================================"
echo "Starting nometa tests"
echo "========================================"
./nometa_tests.sh

# hash test
echo "========================================"
echo "Starting hash tests"
echo "========================================"
./hash_tests.sh

# rescue tests
echo "========================================"
echo "Starting rescue tests"
echo "========================================"
./rescue_tests.sh

echo "========================================"
echo "Starting rescue with specified range tests"
echo "========================================"
./rescue_from_to_tests.sh

echo "========================================"
echo "Starting rescue with specified uid tests"
echo "========================================"
./rescue_pick_uid_tests.sh

# output file tests
echo "========================================"
echo "Starting output file path logic tests"
echo "========================================"
./out_file_logic_tests.sh

# corruption tests
echo "========================================"
echo "Starting corruption tests"
echo "========================================"
./corruption_tests.sh

exit 1
