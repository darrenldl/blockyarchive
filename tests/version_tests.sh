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

# Check all of them
for ver in ${VERSIONS[*]}; do
    echo -n "Checking version $ver container"
    output=$(./rsbx check --json --verbose dummy$ver.sbx)
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
    echo -n "Checking show output for $ver container"
    output=$(./rsbx show --json dummy$ver.sbx)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
        echo " ==> Invalid JSON"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".blocks[0].sbxContainerVersion") == $ver ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo -n "Decoding version $ver container"
  output=$(./rsbx decode --json --verbose -f dummy$ver.sbx dummy$ver)
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

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo -n "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
