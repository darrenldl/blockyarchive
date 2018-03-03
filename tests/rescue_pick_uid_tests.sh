#!/bin/bash

echo "Generating random uids"
uid1=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid2=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid3=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid_unused=$(cat /dev/urandom | tr -dc 0-9a-f | fold -w 12 | head -n 1)

./osbx encode --uid $uid1 -f dummy rescue_picky_uid1.sbx
./osbx encode --uid $uid2 -f dummy rescue_picky_uid2.sbx
./osbx encode --uid $uid3 -f dummy rescue_picky_uid3.sbx

# String everything together
echo "Crafting dummy disk 2 file"
rm dummy_disk2
cat rescue_picky_uid1.sbx >> dummy_disk2
cat rescue_picky_uid2.sbx >> dummy_disk2
cat rescue_picky_uid3.sbx >> dummy_disk2

# Rescue from disk
echo "Rescuing from dummy disk 2 with unused uid"
rm -rf rescued_data2
mkdir rescued_data2
./osbx rescue --only-pick-uid $uid_unused dummy_disk2 rescued_data2
if [ ! -f "rescued_data2/"$uid1 ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
fi
if [ ! -f "rescued_data2/"$uid2 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
fi
if [ ! -f "rescued_data2/"$uid3 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
fi

echo "Rescuing from dummy disk 2 with "$uid1
./osbx rescue --only-pick-uid $uid1 dummy_disk2 rescued_data2
if [ -f "rescued_data2/"$uid1 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
fi
if [ ! -f "rescued_data2/"$uid2 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
fi
if [ ! -f "rescued_data2/"$uid3 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
fi
echo""

echo "Decoding rescued file"
./osbx decode "rescued_data2/"$uid1 "rescued_data2/"$uid1.decoded
echo""

echo "Comparing decoded data to original"
cmp dummy "rescued_data2/"$uid1.decoded
if [[ $? == 0 ]]; then
    echo "No mismatches detected"
else
    echo "Mismatch detected"
fi
