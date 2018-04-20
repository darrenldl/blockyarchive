#!/bin/bash

exit_code=0

VERSIONS=(1 2 3)

# Encode in all 3 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  output=$(./rsbx encode --json --sbx-version $ver -f --no-meta dummy dummy$ver.sbx 2>/dev/null)
  if [[ $(echo $output | jq -r ".stats.sbxVersion") != "$ver" ]]; then exit_code=1; fi
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  output=$(./rsbx decode --json -f dummy$ver.sbx dummy$ver 2>/dev/null)
  if [[ $(echo $output | jq -r ".stats.sbxVersion") != "$ver" ]]; then exit_code=1; fi
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver &>/dev/null
  if [[ $? == 0 ]]; then
    echo "==> NOT okay"
    exit_code=1
  else
    echo "==> Okay"
  fi
done

exit $exit_code
