#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

corrupt_count=10

for ver in ${VERSIONS[*]}; do
  for (( i=0; i < 3; i++ )); do
    echo -n "Encoding in version $ver"
    output=$(./rsbx encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
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

    echo "Corrupting at $corrupt_count random positions"
    for (( p=0; p < $corrupt_count; p++ )); do
      pos=$((RANDOM % $file_size))
      # echo "#$p corruption, corrupting byte at position : $pos"
      corrupt $pos $container_name
    done

    echo -n "Decoding version $ver container"
    output=$(./rsbx decode --json --verbose dummy$ver.sbx dummy$ver -f)
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

    echo -n "Decoding version $ver container (stdout output)"
    output=$(./rsbx decode --json --verbose dummy$ver.sbx - 2>&1 > dummy"$ver"_stdout)
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

    echo -n "Comparing decode output file and stdout output"
    cmp dummy$ver dummy"$ver"_stdout
    if [[ $? == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi
  done
done

exit $exit_code
