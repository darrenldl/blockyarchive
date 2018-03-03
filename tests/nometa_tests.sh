#!/bin/bash

VERSIONS=(1 2 3)

# Encode in all 3 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  ./osbx encode --sbx-version $ver -f --no-meta dummy dummy$ver.sbx
  echo ""
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  ./osbx decode -f dummy$ver.sbx dummy$ver
  echo ""
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver
  if [[ $? == 0 ]]; then
    echo "No mismatches detected"
  else
    echo "Mismatch detected"
  fi
done
