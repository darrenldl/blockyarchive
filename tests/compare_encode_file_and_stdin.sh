#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(./rsbx encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
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
             ./rsbx encode --json --sbx-version $ver -f - dummy"$ver"_stdin.sbx \
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

for ver in ${VERSIONS[*]}; do
  echo -n "Decoding version $ver container (stdin input)"
  output=$(./rsbx decode --json --verbose -f dummy$ver.sbx dummy"$ver"_stdin)
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

for ver in ${VERSIONS[*]}; do
  echo -n "Comparing decoded version $ver container data to original (stdin input)"
  cmp dummy dummy"$ver"_stdin
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
