#!/bin/bash

uid=$(cat /dev/urandom | tr -dc 0-9a-f | fold -w 12 | head -n 1)

dd if=/dev/urandom of=dummy bs=2096 count=1 &>/dev/null

rm dummy_official.sbx

python3 SeqBox/sbxenc.py -uid $uid dummy dummy_official.sbx -o &> /dev/null

rm dummy_osbx.sbx

./osbx encode dummy dummy_osbx.sbx -f --uid $uid -s 1 &> /dev/null

cmp -i 512 dummy_official.sbx dummy_osbx.sbx

if [[ $? == 0 ]]; then
  echo "No mismatches found"
else
  echo "Mismatches detected"
fi
