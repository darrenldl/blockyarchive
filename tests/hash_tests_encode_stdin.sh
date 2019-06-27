#!/bin/bash

exit_code=0

HASHES=("sha1" "sha256" "sha512")
if [[ $(command -v b2sum) != "" ]]; then
    HASHES[3]="blake2b-256"
    HASHES[4]="blake2b-512"
fi

# Record the hashes
a[0]="SHA1 - "$(sha1sum   dummy | awk '{print $1}')
a[1]="SHA256 - "$(sha256sum dummy | awk '{print $1}')
a[2]="SHA512 - "$(sha512sum dummy | awk '{print $1}')
if [[ $(command -v b2sum) != "" ]]; then
    a[3]="BLAKE2b-256 - "$(b2sum -l 256 dummy | awk '{print $1}')
    a[4]="BLAKE2b-512 - "$(b2sum        dummy | awk '{print $1}')
fi

# Encode in all 4 hashes
i=0
for h in ${HASHES[*]}; do
  echo -n "Encoding in hash $h"
  output=$(cat dummy | \
             ./../blkar encode --json --hash $h -f - dummy$h.sbx )
  hash=$(echo $output | jq -r ".stats.hash")
  if [[ $(echo $output | jq -r ".error") != "null" ]]; then
      echo "Invalid JSON"
      exit_code=1
  fi
  if [[ $hash == ${a[$i]} ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
  i=$[$i + 1]
done

# Check all of them
i=0
for h in ${HASHES[*]}; do
  echo -n "Checking hash $h container"
  output=$(./../blkar check --json --verbose dummy$h.sbx)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
  if [[ $(echo $output | jq -r ".stats.numberOfBlocksFailedCheck") == 0 ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
done

# Decode all of them
i=0
for h in ${HASHES[*]}; do
  echo -n "Decoding hash $h container"
  output=$(./../blkar decode --json -f dummy$h.sbx dummy$h)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
  fi
  recorded_hash=$(echo $output | jq -r ".stats.recordedHash")
  output_file_hash=$(echo $output | jq -r ".stats.hashOfOutputFile")
  if [[ $recorded_hash == ${a[$i]} ]]; then
      echo -n " ==> Okay"
  else
      echo -n " ==> NOT okay"
      exit_code=1
  fi
  if [[ $output_file_hash == ${a[$i]} ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
  i=$[$i + 1]
done

# Compare to original file
for h in ${HASHES[*]}; do
  echo -n "Comparing decoded hash $h container data to original"
  cmp dummy dummy$h
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

echo $exit_code > exit_code
