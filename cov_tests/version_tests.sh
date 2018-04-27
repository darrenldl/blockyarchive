#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

VERSIONS=(1 2 3 17 18 19)

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  kcov_rsbx encode --sbx-version $ver -f dummy dummy$ver.sbx \
            --rs-data 10 --rs-parity 2 &>/dev/null
done

# Check all
for ver in ${VERSIONS[*]}; do
    echo "Checking version $ver container"
    output=$(kcov_rsbx check --json --verbose dummy$ver.sbx)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
        echo " ==> Invalid JSON"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi
done

# Show all
for ver in ${VERSIONS[*]}; do
    echo "Checking show output for $ver container"
    output=$(kcov_rsbx check --json dummy$ver.sbx)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
        echo " ==> Invalid JSON"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.sbxContainerVersion") == $ver ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  kcov_rsbx decode -f dummy$ver.sbx dummy$ver &>/dev/null
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
