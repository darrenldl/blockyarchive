#!/bin/bash

exit_code=0

echo "Creating empty files"
touch dummy_empty1
touch dummy_empty2

echo "Encoding files"
output=$(./rsbx encode --json -f dummy_empty1 --uid DEADBEEF0001 2>/dev/null)
if [[ $(echo $output | jq -r ".stats.fileUID") != "DEADBEEF0001" ]]; then
    echo "Invalid JSON"
    exit_code=1
fi
output=$(./rsbx encode --json -f dummy_empty2 --uid DEADBEEF0002 2>/dev/null)
if [[ $(echo $output | jq -r ".stats.fileUID") != "DEADBEEF0002" ]]; then
    echo "Invalid JSON"
    exit_code=1
fi

echo "Crafting dummy disk file"
rm dummy_empty_disk &>/dev/null
cat dummy_empty1.sbx >> dummy_empty_disk
cat dummy_empty2.sbx >> dummy_empty_disk

echo "Rescuing from dummy disk"

echo "Checking that rsbx only decodes first block"
rm rescued_data/DEADBEEF* &>/dev/null
output=$(./rsbx rescue --json dummy_empty_disk rescued_data --from 0 --to 511 2>/dev/null)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo "Invalid JSON"
    exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0001" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data/DEADBEEF0002" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
  exit_code=1
fi

echo "Checking that rsbx only decodes second block"
rm rescued_data/DEADBEEF* &>/dev/null
output=$(./rsbx rescue --json dummy_empty_disk rescued_data --from 512 --to 512 2>/dev/null)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo "Invalid JSON"
    exit_code=1
fi
if [ ! -f "rescued_data/DEADBEEF0001" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
  exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0002" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
  exit_code=1
fi

echo "Checking that rsbx decodes both blocks"
rm rescued_data/DEADBEEF* &>/dev/null
output=$(./rsbx rescue --json dummy_empty_disk rescued_data 2>/dev/null)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo "Invalid JSON"
    exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0001" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
  exit_code=1
fi
if [ -f "rescued_data/DEADBEEF0002" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
  exit_code=1
fi

exit $exit_code
