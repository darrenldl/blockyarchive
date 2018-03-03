#!/bin/bash

HASHES=("sha1" "sha256" "sha512" "blake2b-512")

# Encode in all 4 hashes
for h in ${HASHES[*]}; do
  echo "Encoding in hash $h"
  ./osbx encode --hash $h -f dummy dummy$h.sbx
  echo ""
done

# Decode all of them
for h in ${HASHES[*]}; do
  echo "Decoding hash $h container"
  ./osbx decode -f dummy$h.sbx dummy$h
  echo ""
done

# Compare to original file
for h in ${HASHES[*]}; do
  echo "Comparing decoded hash $h container data to original"
  cmp dummy dummy$h
  if [[ $? == 0 ]]; then
    echo "No mismatches detected"
  else
    echo "Mismatch detected"
  fi
done
