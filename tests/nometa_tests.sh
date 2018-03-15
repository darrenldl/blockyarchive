#!/bin/bash

VERSIONS=(1 2 3)

# Encode in all 3 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  ./rsbx encode --sbx-version $ver -f --no-meta dummy dummy$ver.sbx &>/dev/null
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  ./rsbx decode -f dummy$ver.sbx dummy$ver &>/dev/null
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver &>/dev/null
  if [[ $? == 0 ]]; then
    echo "==> NOT okay"
  else
    echo "==> Okay"
  fi
done
