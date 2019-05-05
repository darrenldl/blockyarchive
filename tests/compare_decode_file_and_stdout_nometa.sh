#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

corrupt() {
  dd if=/dev/zero of=$2 bs=10 count=1 seek=$1 conv=notrunc &>/dev/null
}

file_size=$(ls -l dummy | awk '{ print $5 }')

corrupt_count=10

for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(./../blkar encode --json --sbx-version $ver -f --no-meta dummy dummy$ver.sbx \
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

  container_name=dummy$ver.sbx

  echo "Decoding version $ver container"
  output=$(./../blkar decode --json --verbose dummy$ver.sbx dummy$ver -f)
  # if [[ $(echo $output | jq -r ".error") != null ]]; then
  #   echo " ==> Invalid JSON"
  #   exit_code=1
  # fi
  # if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
  #   echo " ==> Okay"
  # else
  #   echo " ==> NOT okay"
  #   exit_code=1
  # fi

  echo "Decoding version $ver container (stdout output)"
  output=$(./../blkar decode --json --verbose dummy$ver.sbx - 2>&1 > dummy"$ver"_stdout)
  # if [[ $(echo $output | jq -r ".error") != null ]]; then
  #   echo " ==> Invalid JSON"
  #   exit_code=1
  # fi
  # if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
  #   echo " ==> Okay"
  # else
  #   echo " ==> NOT okay"
  #   exit_code=1
  # fi

  echo -n "Comparing decode output file and stdout output"
  cmp dummy$ver dummy"$ver"_stdout
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

echo $exit_code > exit_code
