#!/bin/bash

exit_code=0

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

for (( i=0; i < 10; i++ )); do
  from=$[RANDOM % 1000]
  to=$[RANDOM % 100000 + 1001]
  echo -n "Encoding from $from to $to"
  output=$(./../blkar encode --json -f dummy --from $from --to-exc $to)
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.fileSize") == $[to - from] ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi

  echo "Decoding"
  output=$(./../blkar decode --json -f dummy.sbx data_chunk)
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
  fi

  echo -n "Checking if output data chunk matches the original file portion"
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
  fi
  rm -f data_chunk_orig
  dd if=dummy of=data_chunk_orig bs=1 count=$[to - from] skip=$from 2>/dev/null
  cmp data_chunk data_chunk_orig
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

echo $exit_code > exit_code
