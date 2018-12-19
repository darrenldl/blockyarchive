#!/bin/bash

exit_code=0

echo "Generating random uids"
uid1=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid2=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid3=$(cat /dev/urandom | tr -dc 0-9A-F | fold -w 12 | head -n 1)
uid_unused=$(cat /dev/urandom | tr -dc 0-9a-f | fold -w 12 | head -n 1)

echo -n "Encoding files"
output=$(./blkar encode --json --uid $uid1 -f dummy rescue_picky_uid1.sbx)
if [[ $(echo $output | jq -r ".stats.fileUID") == "$uid1" ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
output=$(./blkar encode --json --uid $uid2 -f dummy rescue_picky_uid2.sbx)
if [[ $(echo $output | jq -r ".stats.fileUID") == "$uid2" ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
output=$(./blkar encode --json --uid $uid3 -f dummy rescue_picky_uid3.sbx)
if [[ $(echo $output | jq -r ".stats.fileUID") == "$uid3" ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

# String everything together
echo "Crafting dummy disk 2 file"
rm dummy_disk2 &>/dev/null
cat rescue_picky_uid1.sbx >> dummy_disk2
cat rescue_picky_uid2.sbx >> dummy_disk2
cat rescue_picky_uid3.sbx >> dummy_disk2

# Rescue from disk
echo -n "Rescuing from dummy disk 2 with unused uid"
rm -rf rescued_data2 &>/dev/null
mkdir rescued_data2 &>/dev/null
output=$(./blkar rescue --json --only-pick-uid $uid_unused dummy_disk2 rescued_data2)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [ ! -f "rescued_data2/"$uid1 ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data2/"$uid2 ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data2/"$uid3 ]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
  exit_code=1
fi

echo -n "Rescuing from dummy disk 2 with "$uid1
output=$(./blkar rescue --json --only-pick-uid $uid1 dummy_disk2 rescued_data2)
if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi
if [ -f "rescued_data2/"$uid1 ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data2/"$uid2 ]; then
  echo -n " ==> Okay"
else
  echo -n " ==> NOT okay"
  exit_code=1
fi
if [ ! -f "rescued_data2/"$uid3 ]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

echo "Decoding rescued file"
output=$(./blkar decode --json "rescued_data2/"$uid1 - 2>&1 > "rescued_data2/"$uid1.decoded)
if [[ $(echo $output | jq -r ".stats.fileUID") != "$uid1" ]]; then
  echo " ==> Invalid JSON"
  exit_code=1
fi

echo -n "Comparing decoded data to original"
cmp dummy "rescued_data2/"$uid1.decoded
if [[ $? == 0 ]]; then
  echo " ==> Okay"
else
  echo " ==> NOT okay"
  exit_code=1
fi

exit $exit_code
