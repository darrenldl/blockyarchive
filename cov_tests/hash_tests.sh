#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

HASHES=("sha1" "sha256" "sha512" "blake2b-512")

# Record the hashes
a[0]=$(sha1sum   dummy | awk '{print $1}')
#a[1]=$(sha256sum dummy | awk '{print $1}')
#a[2]=$(sha512sum dummy | awk '{print $1}')
#a[3]=$(b2sum     dummy | awk '{print $1}')

# Encode in all 4 hashes
i=0
for h in ${HASHES[*]}; do
  echo "Encoding in hash $h"
  output=$(kcov_rsbx encode --hash $h -f dummy dummy$h.sbx | grep "${a[$i]}" )
  if [[ $output == "" ]]; then
      echo "==> NOT okay"
      exit_code=1
  else
      echo "==> Okay"
  fi
  i=$[$i + 1]
done

# Decode all of them
for h in ${HASHES[*]}; do
  echo "Decoding hash $h container"
  kcov_rsbx decode -f dummy$h.sbx dummy$h &>/dev/null
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
