#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

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

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
  echo -n "Encoding in version $ver"
  output=$(./../blkar encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
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

# Show all
for ver in ${VERSIONS[*]}; do
    echo -n "Checking show output for $ver container"
    output=$(./../blkar show --json --pv 1 dummy$ver.sbx 2>/dev/null)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
        echo " ==> Invalid JSON"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".blocks[0].sbxContainerVersion") == $ver ]]; then
        echo -n " ==> Okay"
    else
        echo -n " ==> NOT okay"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".blocks[0].hash") == ${a[1]} ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi
done

new_fnm={}

# Change file name
for ver in ${VERSIONS[*]}; do
    echo -n "Changing file name of "dummy$ver.sbx
    output=$(./../blkar update --json -y --no-hsh -v dummy$ver.sbx)
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
    if [[ $(echo $output | jq -r ".metadataChanges[0].changes[0].field") == "HSH" ]]; then
        echo -n " ==> Okay"
    else
        echo -n " ==> NOT okay"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".metadataChanges[0].changes[0].from") == ${a[1]} ]]; then
        echo -n " ==> Okay"
    else
        echo -n " ==> NOT okay"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".metadataChanges[0].changes[0].to") == "null" ]]; then
        echo -n " ==> Okay"
    else
        echo -n " ==> NOT okay"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".metadataChanges[3]") == "null" ]]; then
        echo -n " ==> Okay"
    else
        echo -n " ==> NOT okay"
        exit_code=1
    fi
    if [[ $(echo $output | jq -r ".metadataChanges[0].changes[1]") == "null" ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi
done

# Show all
for ver in ${VERSIONS[*]}; do
  echo -n "Checking show output for $ver container"
  output=$(./../blkar show --json --pv 1 dummy$ver.sbx 2>/dev/null)
  if [[ $(echo $output | jq -r ".error") != null ]]; then
    echo " ==> Invalid JSON"
    exit_code=1
  fi
  if [[ $(echo $output | jq -r ".blocks[0].sbxContainerVersion") == $ver ]]; then
    echo -n " ==> Okay"
  else
    echo -n " ==> NOT okay"
    exit_code=1
  fi
  if [[ $(echo $output | jq -r ".blocks[0].hash") == "null" ]]; then
      echo " ==> Okay"
  else
      echo " ==> NOT okay"
      exit_code=1
  fi
done

echo $exit_code > exit_code
