#!/bin/bash

source kcov_blkar_fun.sh

exit_code=0

VERSIONS=(1 3 18)

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(kcov_blkar encode --json --sbx-version $ver -f dummy rescue$ver.sbx \
                     --rs-data 10 --rs-parity 2)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
done

rescue1uid=$(kcov_blkar show --json rescue1.sbx | jq -r ".blocks[0].fileUID")
rescue3uid=$(kcov_blkar show --json rescue3.sbx | jq -r ".blocks[0].fileUID")
rescue18uid=$(kcov_blkar show --json rescue18.sbx | jq -r ".blocks[0].fileUID")

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
cat rescue18.sbx >> dummy_disk
cat filler3      >> dummy_disk

# Rescue from the disk
echo "Rescuing from dummy disk"
rm -rf rescued_data &>/dev/null
mkdir rescued_data &>/dev/null
rm rescue_log &>/dev/null
output=$(kcov_blkar rescue --json dummy_disk rescued_data rescue_log)
if [[ $(echo $output | jq -r ".error") != "null" ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
fi

# Check if original bytes were used
echo -n "Checking if original bytes from containers were used"
cmp rescued_data/"$rescue1uid" rescue1.sbx
if [[ $? == 0 ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
cmp rescued_data/"$rescue3uid" rescue3.sbx
if [[ $? == 0 ]]; then
    echo -n " ==> Okay"
else
    echo -n " ==> NOT okay"
    exit_code=1
fi
cmp rescued_data/"$rescue18uid" rescue18.sbx
if [[ $? == 0 ]]; then
    echo " ==> Okay"
else
    echo " ==> NOT okay"
    exit_code=1
fi

# Try to decode the rescued data
echo "Decoding all rescued data"
FILES=rescued_data/*
for f in $FILES; do
  output=$(kcov_blkar decode --json $f $f.decoded)
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
done

echo -n "Comparing decoded data to original"
FILES=rescued_data/*.decoded
for f in $FILES; do
  echo -n "Comparing file $f to original"
  cmp dummy $f
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
