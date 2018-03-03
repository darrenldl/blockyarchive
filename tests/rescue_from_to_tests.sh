#!/bin/bash

echo "Creating empty files"
touch dummy_empty1
touch dummy_empty2
echo ""

echo "Encoding files"
./osbx encode -f dummy_empty1 --uid DEADBEEF0001
./osbx encode -f dummy_empty2 --uid DEADBEEF0002
echo ""

echo "Crafting dummy disk file"
rm dummy_empty_disk
cat dummy_empty1.sbx >> dummy_empty_disk
cat dummy_empty2.sbx >> dummy_empty_disk
echo ""

echo "Rescuing from dummy disk"
echo ""

echo "Checking that osbx only decodes first block"
rm rescued_data/DEADBEEF*
./osbx rescue dummy_empty_disk rescued_data --from 0 --to 511
if [ -f "rescued_data/DEADBEEF0001" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
fi
if [ ! -f "rescued_data/DEADBEEF0002" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
fi

echo "Checking that osbx only decodes second block"
rm rescued_data/DEADBEEF*
./osbx rescue dummy_empty_disk rescued_data --from 512 --to 512
if [ ! -f "rescued_data/DEADBEEF0001" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
fi
if [ -f "rescued_data/DEADBEEF0002" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
fi

echo "Checking that osbx decodes both blocks"
rm rescued_data/DEADBEEF*
./osbx rescue dummy_empty_disk rescued_data
if [ -f "rescued_data/DEADBEEF0001" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
fi
if [ -f "rescued_data/DEADBEEF0002" ]; then
  echo "===> Okay"
else
  echo "===> NOT okay"
fi
