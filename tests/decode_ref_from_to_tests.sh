#!/bin/bash

exit_code=0

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

offset=$[1 + RANDOM % 100]

echo -n "Encoding"
output=$(./../blkar encode --json -f dummy --uid DEADBEEF0001 --sbx-version 17 --rs-data 3 --rs-parity 2)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.fileUID") == "DEADBEEF0001" ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

echo -n "Decoding dummy disk"

output=$(./../blkar decode -f --json dummy.sbx)
if [[ $(echo $output | jq -r ".error") == "null" ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt 0 dummy.sbx

output=$(./../blkar decode -f --json dummy.sbx --ref-to-inc 0)
if [[ $(echo $output | jq -r ".error") == "Error : Failed to find reference block" ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

output=$(./../blkar decode -f --json --ref-from 512 dummy.sbx)
if [[ $(echo $output | jq -r ".error") == "null" ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
