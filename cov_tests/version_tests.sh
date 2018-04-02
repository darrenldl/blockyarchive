#!/bin/bash

# source kcov_rsbx_fun.sh

exit_code=0

TARGET=$HOME/kcov

if [[ $TRAVIS == true ]]; then
    echo "Running on travis"
    export PATH=$TARGET/bin:$PATH
fi

# VERSIONS=(1 2 3 17 18 19)
VERSIONS=(1)

mkdir -p "../target/cov/rsbx"

echo $(which kcov)
echo $PWD

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  #kcov_rsbx encode --sbx-version $ver -f dummy dummy$ver.sbx \
  #          --rs-data 10 --rs-parity 2 #&>/dev/null
  kcov --include-path .. --verify "../target/cov/rsbx" rsbx encode --sbx-version $ver -f dummy dummy$ver.sbx \
                --rs-data 10 --rs-parity 2 #&>/dev/null
  # ./rsbx encode --sbx-version $ver -f dummy dummy$ver.sbx \
  #     --rs-data 10 --rs-parity 2 #&>/dev/null
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  # kcov_rsbx decode -f dummy$ver.sbx dummy$ver &>/dev/null
  ./rsbx decode -f dummy$ver.sbx dummy$ver &>/dev/null
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver
  if [[ $? == 0 ]]; then
    echo "==> Okay"
  else
    echo "==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
