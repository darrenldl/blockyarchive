#!/bin/bash

exit_code=0

VERSIONS=(17 18 19)

# Encode in all 3 RS enabled versions
for ver in ${VERSIONS[*]}; do
  for (( i=0; i < 3; i++ )); do
    burst=$((1001 + RANDOM % 500))
    echo -n "Encoding in version $ver, burst error resistance level $burst"
    output=$(cat dummy |
               ./blkar encode --json --sbx-version $ver -f - dummy$ver.sbx \
                       --rs-data 10 --rs-parity 2 --burst $burst)
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

    # Check all of them
    echo -n "Checking version $ver container"
    output=$(./blkar check --json --pv 2 --verbose dummy$ver.sbx 2>/dev/null)
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

    # Show
    echo -n "Checking show output for $ver container"
    output=$(./blkar show --json --pv 1 dummy$ver.sbx 2>/dev/null)
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

    # Decode
    echo -n "Decoding version $ver container"
    output=$(./blkar decode --json --verbose -f dummy$ver.sbx dummy$ver)
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
    if [[ $(echo $output | jq -r ".stats.recordedHash") == $(echo $output | jq -r ".stats.hashOfOutputFile") ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    # Compare to original file
    echo -n "Comparing decoded version $ver container data to original"
    cmp dummy dummy$ver
    if [[ $? == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi
  done
done

exit $exit_code
