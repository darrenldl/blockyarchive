#!/bin/bash

VERSIONS=(1 2 3 17 18 19)

# Encode in all 3 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  ./rsbx encode --sbx-version $ver -f dummy dummy$ver.sbx \
         --rs-data 10 --rs-parity 2 &>/dev/null
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  ./rsbx decode -f dummy$ver.sbx dummy$ver &>/dev/null
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver
  if [[ $? == 0 ]]; then
    echo "==> No mismatches detected"
  else
    echo "==> Mismatch detected"
  fi
done
