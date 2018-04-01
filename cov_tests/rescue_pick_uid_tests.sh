#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

echo "Generating random uids"
uid1=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid2=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid3=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid_unused=$(cat /dev/urandom | tr -dc 0-9a-f | fold -w 12 | head -n 1)

kcov_rsbx encode --uid $uid1 -f dummy rescue_picky_uid1.sbx &>/dev/null
kcov_rsbx encode --uid $uid2 -f dummy rescue_picky_uid2.sbx &>/dev/null
kcov_rsbx encode --uid $uid3 -f dummy rescue_picky_uid3.sbx &>/dev/null

# String everything together
echo "Crafting dummy disk 2 file"
rm dummy_disk2 &>/dev/null
cat rescue_picky_uid1.sbx >> dummy_disk2
cat rescue_picky_uid2.sbx >> dummy_disk2
cat rescue_picky_uid3.sbx >> dummy_disk2

# Rescue from disk
echo "Rescuing from dummy disk 2 with unused uid"
rm -rf rescued_data2 &>/dev/null
mkdir rescued_data2 &>/dev/null
kcov_rsbx rescue --only-pick-uid $uid_unused dummy_disk2 rescued_data2 &>/dev/null
if [ ! -f "rescued_data2/"$uid1 ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data2/"$uid2 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
    exit_code=1
fi
if [ ! -f "rescued_data2/"$uid3 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
    exit_code=1
fi

echo "Rescuing from dummy disk 2 with "$uid1
kcov_rsbx rescue --only-pick-uid $uid1 dummy_disk2 rescued_data2 &>/dev/null
if [ -f "rescued_data2/"$uid1 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
    exit_code=1
fi
if [ ! -f "rescued_data2/"$uid2 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
    exit_code=1
fi
if [ ! -f "rescued_data2/"$uid3 ]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
    exit_code=1
fi

echo "Decoding rescued file"
kcov_rsbx decode "rescued_data2/"$uid1 "rescued_data2/"$uid1.decoded &>/dev/null

echo "Comparing decoded data to original"
cmp dummy "rescued_data2/"$uid1.decoded
if [[ $? == 0 ]]; then
    echo "==> Okay"
else
    echo "==> NOT okay"
    exit_code=1
fi

exit $exit_code
