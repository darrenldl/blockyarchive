#!/bin/bash

exit_code=0

offset=$[1 + RANDOM % 100]

echo "Creating empty files"
touch dummy_empty1
touch dummy_empty2

echo -n "Encoding 1st file"
output=$(./../blkar encode --sbx-version 1 --json -f dummy_empty1 --uid DEADBEEF0001)
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

echo -n "Encoding 2nd file"
output=$(./../blkar encode --sbx-version 1 --json -f dummy_empty2 --uid DEADBEEF0002)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [[ $(echo $output | jq -r ".stats.fileUID") == "DEADBEEF0002" ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

echo "Crafting dummy disk file"
rm dummy_empty_disk &>/dev/null
tocuh dummy_empty_disk
truncate -s $offset dummy_empty_disk
cat dummy_empty1.sbx >> dummy_empty_disk
cat dummy_empty2.sbx >> dummy_empty_disk

echo "Rescuing from dummy disk"

rm -rf rescued_data &>/dev/null
mkdir rescued_data &>/dev/null

echo -n "Checking that blkar only decodes first block"
rm rescued_data/DEADBEEF* &>/dev/null
output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from $offset --to-inc 511 --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0001" ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data/DEADBEEF0002" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar only decodes second block"
rm rescued_data/DEADBEEF* &>/dev/null
output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from $[offset + 512] --to-inc 512 --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [ ! -f "rescued_data/DEADBEEF0001" ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0002" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo -n "Checking that blkar decodes both blocks"
rm rescued_data/DEADBEEF* &>/dev/null
output=$(./../blkar rescue --json dummy_empty_disk rescued_data --from $offset --force-misalign)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0001" ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0002" ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo $exit_code > exit_code
