#!/bin/bash

exit_code=0

HASHES=("sha1" "sha256" "sha512" "blake2b-512")

# Record the hashes
a[0]=$(sha1sum   dummy | awk '{print $1}')
a[1]=$(sha256sum dummy | awk '{print $1}')
a[2]=$(sha512sum dummy | awk '{print $1}')
if [[ $(command -v b2sum) != "" ]]; then
    a[3]=$(b2sum     dummy | awk '{print $1}')
fi

# Encode in all 4 hashes
i=0
for h in ${HASHES[*]}; do
  echo "Encoding in hash $h"
  output=$(./rsbx encode --json --hash $h -f dummy dummy$h.sbx 2>/dev/null )
  hash=$(echo $output | jq -r ".stats.hash" | awk '{ print $3 }')
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
      echo "Invalid JSON"
      exit_code=1
  fi
  if [[ $hash == ${a[$i]} ]]; then
      echo "==> Okay"
  else
      echo "==> NOT okay"
      exit_code=1
  fi
  i=$[$i + 1]
done

# Decode all of them
i=0
for h in ${HASHES[*]}; do
  echo "Decoding hash $h container"
  output=$(./rsbx decode --json -f dummy$h.sbx dummy$h 2>/dev/null)
  recorded_hash=$(echo $output | jq -r ".stats.recordedHash" | awk '{ print $3 }')
  output_file_hash=$(echo $output | jq -r ".stats.hashOfOutputFile" | awk '{ print $3 }')
  if [[ $recorded_hash == ${a[$i]} ]]; then
      echo "==> Okay"
  else
      echo "==> NOT okay"
      exit_code=1
  fi
  if [[ $output_file_hash == ${a[$i]} ]]; then
      echo "==> Okay"
  else
      echo "==> NOT okay"
      exit_code=1
  fi
  i=$[$i + 1]
done

# Compare to original file
for h in ${HASHES[*]}; do
  echo "Comparing decoded hash $h container data to original"
  cmp dummy dummy$h
  if [[ $? == 0 ]]; then
    echo "==> Okay"
  else
    echo "==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
