#!/bin/bash

COV_DIR="target/cov/total"

if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
    exit 0
fi

wget https://codecov.io/bash -O codecov_uploader
chmod u+x codecov_uploader

./codecov_uploader -s $COV_DIR

echo "Uploaded code coverage to Codecov"

exit 0
