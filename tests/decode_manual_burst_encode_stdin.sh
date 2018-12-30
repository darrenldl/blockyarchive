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

    # Decode without --burst flag
    echo -n "Decoding version $ver container without --burst"
    output=$(./blkar decode --json --verbose -f dummy$ver.sbx dummy$ver)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
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

    # Decode with --burst flag
    echo -n "Decoding version $ver container with --burst"
    output=$(./blkar decode --json --verbose --burst $burst -f dummy$ver.sbx dummy$ver)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
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

echo $exit_code > exit_code
