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

mv dummy.sbx dummy.sbx.tmp
touch dummy.sbx
truncate -s $offset dummy.sbx
cat dummy.sbx.tmp >> dummy.sbx
rm dummy.sbx.tmp

echo -n "Checking dummy disk"

output=$(./../blkar check --json --ref-from $offset --force-misalign dummy.sbx)
if [[ $(echo $output | jq -r ".error") == "null" ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

corrupt $offset dummy.sbx

output=$(./../blkar check --json --ref-from $offset --ref-to-inc $offset --force-misalign dummy.sbx)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi

output=$(./../blkar check --json --ref-from $[offset + 512] --force-misalign dummy.sbx)
if [[ $(echo $output | jq -r ".error") == "null" ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
