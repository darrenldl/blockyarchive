#!/bin/bash

uid=$(cat /dev/urandom | tr -dc 0-9a-f | fold -w 12 | head -n 1)

dd if=/dev/urandom of=dummy bs=2096 count=1 &>/dev/null

rm dummy_official.sbx &>/dev/null

python3 SeqBox/sbxenc.py -uid $uid dummy dummy_official.sbx -o &>/dev/null

rm dummy_rsbx.sbx &>/dev/null

./rsbx encode dummy dummy_rsbx.sbx -f --uid $uid &>/dev/null

cmp -i 512 dummy_official.sbx dummy_rsbx.sbx
if [[ $? == 0 ]]; then
  echo "==> Okay"
else
  echo "==> NOT okay"
  exit 1
fi
