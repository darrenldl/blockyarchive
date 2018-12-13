#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(./rsbx encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
                  --rs-data 10 --rs-parity 2)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo -n "Decoding version $ver container"
  output=$(./rsbx decode --json --verbose dummy$ver.sbx - 2>&1 > dummy$ver)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
