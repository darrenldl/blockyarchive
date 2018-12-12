#!/bin/bash

if ! [[ $TARGET == x86_64-unknown-linux-gnu && $DISABLE_COV == "" ]]; then
    exit 0
fi

export PATH=$HOME/kcov/bin:$PATH

for file in target/debug/rsbx-*; do
    if [[ $file == *.d ]]; then
        continue
    fi

    kcov --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/rsbx" "$file"
done

wget https://codecov.io/bash -O codecov_uploader
chmod u+x codecov_uploader

./codecov_uploader -s "target/cov/rsbx"

echo "Uploaded code coverage to Codecov"

for file in target/debug/rsbx-*; do
    if [[ $file == *.d ]]; then
        continue
    fi

    kcov --coveralls-id=$TRAVIS_JOB_ID --exclude-pattern=/.cargo,/usr/lib --verify "target/cov/rsbx" "$file"
done

echo "Uploaded code coverage to Coveralls"

exit 0
