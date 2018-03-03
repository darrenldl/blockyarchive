#!/bin/bash

VERSIONS=(1 2 3)

# Encode in all 3 versions
for ver in ${VERSIONS[*]}; do
  echo "Encoding in version $ver"
  ./osbx encode --sbx-version $ver -f dummy rescue$ver.sbx
  echo ""
done

# Generate random filler data
echo "Generating random filler data"
dd if=/dev/urandom of=filler1 bs=10240 count=1
dd if=/dev/urandom of=filler2 bs=128   count=1
dd if=/dev/urandom of=filler3 bs=512   count=1
echo ""

# String everything together
echo "Crafting dummy disk file"
rm dummy_disk
cat filler1     >> dummy_disk
cat rescue3.sbx >> dummy_disk
cat filler2     >> dummy_disk
cat rescue1.sbx >> dummy_disk
cat filler3     >> dummy_disk
cat rescue2.sbx >> dummy_disk
cat filler2     >> dummy_disk
cat filler3     >> dummy_disk
echo ""

# Rescue from the disk
echo "Rescuing from dummy disk"
rm -rf rescued_data
mkdir rescued_data
rm rescue_log
./osbx rescue dummy_disk rescued_data rescue_log
echo ""

# Try to decode the rescued data
echo "Decoding all rescued data"
FILES=rescued_data/*
for f in $FILES; do
  ./osbx decode $f $f.decoded
  echo ""
done

echo "Comparing decoded data to original"
FILES=rescued_data/*.decoded
for f in $FILES; do
  echo "Comparing file $f to original"
  cmp dummy $f
  if [[ $? == 0 ]]; then
    echo "No mismatches detected"
  else
    echo "Mismatch detected"
  fi
done
