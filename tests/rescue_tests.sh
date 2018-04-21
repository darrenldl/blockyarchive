#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  output=$(./rsbx encode --json --sbx-version $ver -f dummy rescue$ver.sbx \
                  --rs-data 10 --rs-parity 2 2>/dev/null)
  if [[ $(echo $output | jq -r ".stats.sbxVersion") != "$ver" ]]; then exit_code=1; fi
done

# Generate random filler data
echo "Generating random filler data"
dd if=/dev/urandom of=filler1 bs=10240 count=1 &>/dev/null
dd if=/dev/urandom of=filler2 bs=128   count=1 &>/dev/null
dd if=/dev/urandom of=filler3 bs=512   count=1 &>/dev/null

# String everything together
echo "Crafting dummy disk file"
rm dummy_disk &>/dev/null
cat filler1      >> dummy_disk
cat rescue3.sbx  >> dummy_disk
cat filler2      >> dummy_disk
cat rescue1.sbx  >> dummy_disk
cat filler3      >> dummy_disk
cat rescue2.sbx  >> dummy_disk
cat filler2      >> dummy_disk
cat filler3      >> dummy_disk
cat rescue17.sbx >> dummy_disk
cat filler2      >> dummy_disk
cat rescue19.sbx >> dummy_disk
cat filler2      >> dummy_disk
cat filler3      >> dummy_disk
cat filler3      >> dummy_disk
cat rescue18.sbx >> dummy_disk
cat filler3      >> dummy_disk

# Rescue from the disk
echo "Rescuing from dummy disk"
rm -rf rescued_data &>/dev/null
mkdir rescued_data &>/dev/null
rm rescue_log &>/dev/null
output=$(./rsbx rescue --json dummy_disk rescued_data rescue_log 2>/dev/null)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo "Invalid JSON"
    exit_code=1
fi

# Try to decode the rescued data
echo "Decoding all rescued data"
FILES=rescued_data/*
for f in $FILES; do
  output=$(./rsbx decode --json $f $f.decoded 2>/dev/null)
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
      echo "Invalid JSON"
      exit_code=1
  fi
done

echo "Comparing decoded data to original"
FILES=rescued_data/*.decoded
for f in $FILES; do
  echo "Comparing file $f to original"
  cmp dummy $f
  if [[ $? == 0 ]]; then
    echo "==> Okay"
  else
    echo "==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
