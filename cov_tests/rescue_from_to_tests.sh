#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

echo "Creating empty files"
touch dummy_empty1
touch dummy_empty2

echo "Encoding files"
kcov_rsbx encode -f dummy_empty1 --uid DEADBEEF0001 &>/dev/null
kcov_rsbx encode -f dummy_empty2 --uid DEADBEEF0002 &>/dev/null

echo "Crafting dummy disk file"
rm dummy_empty_disk &>/dev/null
cat dummy_empty1.sbx >> dummy_empty_disk
cat dummy_empty2.sbx >> dummy_empty_disk

echo "Rescuing from dummy disk"

echo "Checking that rsbx only decodes first block"
rm rescued_data/DEADBEEF* &>/dev/null
kcov_rsbx rescue dummy_empty_disk rescued_data --from 0 --to 511 &>/dev/null
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
kcov_rsbx rescue dummy_empty_disk rescued_data --from 512 --to 512 &>/dev/null
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
kcov_rsbx rescue dummy_empty_disk rescued_data &>/dev/null
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
