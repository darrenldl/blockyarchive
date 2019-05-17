#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

source functions.sh

# Encode in all 6 versions
for ver in ${VERSIONS[*]}; do
    for c in 0 1; do
        echo -n "Generating container of version $ver, copy #$c"
        corrupt 1000 dummy
        output=$(./../blkar encode --uid DEADBEEF0123 --json --sbx-version $ver -f dummy dummy$ver.$c.sbx \
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
done

# Check hash of all containers
for ver in ${VERSIONS[*]}; do
    if   [[ $ver == 1 || $ver == 17 ]]; then
        block_size=512
    elif [[ $ver == 2 || $ver == 18 ]]; then
        block_size=128
    elif [[ $ver == 3 || $ver == 19 ]]; then
        block_size=4096
    fi

    echo -n "Checking container version $ver, copy #0"
    output=$(./../blkar check --hash-only --json --pv 2 --verbose dummy$ver.0.sbx 2>/dev/null)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
        echo " ==> Invalid JSON"
        exit_code=1
    fi
    recorded_hash=$(echo $output | jq -r ".stats.recordedHash" | awk '{ print $3 }')
    stored_data_hash=$(echo $output | jq -r ".stats.hashOfStoredData" | awk '{ print $3 }')
    if [[ $recorded_hash == $stored_data_hash ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi

    echo "Overwriting first block of copy #0 with copy #1"
    dd if=dummy$ver.1.sbx of=dummy$ver.0.sbx bs=$block_size count=1 conv=notrunc &>/dev/null

    echo -n "Checking container version $ver, copy #0 after overwrite"
    output=$(./../blkar check --hash-only --json --pv 2 --verbose dummy$ver.0.sbx 2>/dev/null)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
        echo " ==> Invalid JSON"
        exit_code=1
    fi
    recorded_hash=$(echo $output | jq -r ".stats.recordedHash" | awk '{ print $3 }')
    stored_data_hash=$(echo $output | jq -r ".stats.hashOfStoredData" | awk '{ print $3 }')
    if [[ $recorded_hash != $stored_data_hash ]]; then
        echo " ==> Okay"
    else
        echo " ==> NOT okay"
        exit_code=1
    fi
done

echo $exit_code > exit_code
