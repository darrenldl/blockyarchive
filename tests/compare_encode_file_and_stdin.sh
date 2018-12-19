#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(./blkar encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
                  --rs-data 10 --rs-parity 2 --uid DEADBEEF0123)
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

for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver using stdin as input"
  output=$(cat dummy | \
             ./blkar encode --json --sbx-version $ver -f - dummy"$ver"_stdin.sbx \
                    --rs-data 10 --rs-parity 2 --uid DEADBEEF0123)
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

# Compare containers
for ver in ${VERSIONS[*]}; do
  echo -n "Comparing two containers of version $ver"
  if   (( $ver <= 3 )); then
    skip=128
  elif (( $ver == 17 )); then
    skip=$[3 * 512]
  elif (( $ver == 18 )); then
    skip=$[3 * 128]
  else
    skip=$[3 * 4096]
  fi

  cmp -i $skip dummy$ver.sbx dummy"$ver"_stdin.sbx
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
