#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(./blkar encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
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

# Create corrupted copies
for ver in ${VERSIONS[*]}; do
  cp dummy$ver.sbx dummy$ver.1.sbx
  cp dummy$ver.sbx dummy$ver.2.sbx
  cp dummy$ver.sbx dummy$ver.3.sbx
  cp dummy$ver.sbx dummy$ver.4.sbx
  mv dummy$ver.sbx dummy$ver.5.sbx

  corrupt  5000 dummy$ver.1.sbx
  corrupt 10000 dummy$ver.1.sbx
  corrupt 15000 dummy$ver.1.sbx
  corrupt 20000 dummy$ver.1.sbx

  corrupt 10000 dummy$ver.2.sbx
  corrupt 15000 dummy$ver.2.sbx
  corrupt 20000 dummy$ver.2.sbx

  corrupt 15000 dummy$ver.3.sbx
  corrupt 20000 dummy$ver.3.sbx

  corrupt 20000 dummy$ver.4.sbx

  corrupt  5000 dummy$ver.5.sbx
  corrupt 10000 dummy$ver.5.sbx
  corrupt 15000 dummy$ver.5.sbx
done

# Decode all of them
for ver in ${VERSIONS[*]}; do
  echo "Decoding version $ver container"
  rm -f dummy$ver
  for i in 1 2 3 4 5; do
    echo -n "  pass $i"
    output=$(./blkar decode --json --verbose --multi-pass dummy$ver.$i.sbx dummy$ver)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $i < 5 ]]; then
      if [[ $(echo $output | jq -r ".stats.recordedHash") != $(echo $output | jq -r ".stats.hashOfOutputFile") ]]; then
        echo " ==> Okay"
      else
        echo " ==> NOT okay"
        exit_code=1
      fi
    else
      if [[ $(echo $output | jq -r ".stats.recordedHash") == $(echo $output | jq -r ".stats.hashOfOutputFile") ]]; then
        echo " ==> Okay"
      else
        echo " ==> NOT okay"
        exit_code=1
      fi
    fi
  done
done

# Compare to original file
for ver in ${VERSIONS[*]}; do
  echo -n "Comparing decoded version $ver container data to original"
  cmp dummy dummy$ver
  if [[ $? == 0 ]]; then
    echo " ==> Okay"
  else
    echo " ==> NOT okay"
    exit_code=1
  fi
done

exit $exit_code
