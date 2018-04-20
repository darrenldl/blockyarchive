#!/bin/bash

exit_code=0

VERSIONS=(17 18 19)

corrupt() {
    dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

file_size=$[1024 * 1024 * 1]

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 3; i++ )); do
        if   [[ $ver == 17 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver == 18 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        else
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        fi

        container_name=corrupt_$data_shards\_$parity_shards\_$ver.sbx

        echo "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
        output=$(./rsbx encode --json --sbx-version $ver -f dummy $container_name \
                        --hash sha1 \
                        --rs-data $data_shards --rs-parity $parity_shards 2>/dev/null)
        if [[ $(echo $output | jq -r ".stats.sbxVersion") != "$ver" ]]; then
            echo "Invalid JSON"
            exit_code=1
        fi
        if [[ $(echo $output | jq -r ".stats.hash" | awk '{ print $1 }') != "SHA1" ]]; then
            echo "Invalid JSON"
            exit_code=1
        fi

        echo "Corrupting at $parity_shards random positions"
        for (( p=0; p < $parity_shards; p++ )); do
            pos=$((RANDOM % $file_size))
            # echo "#$p corruption, corrupting byte at position : $pos"
            corrupt $pos $container_name
        done

        echo "Repairing"
        output=$(./rsbx repair --json $container_name 2>/dev/null)
        if [[ $(echo $output | jq -r ".error") != "null" ]]; then
            echo "Invalid JSON"
            exit_code=1
        fi
        if [[ $(echo $output | jq -r ".stats.sbxVersion") != "$ver" ]]; then
            echo "Invalid JSON"
            exit_code=1
        fi

        output_name=dummy_$data_shards\_$parity_shards

        echo "Decoding"
        output=$(./rsbx decode --json -f $container_name $output_name 2>/dev/null)
        if [[ $(echo $output | jq -r ".error") != "null" ]]; then
            echo "Invalid JSON"
            exit_code=1
        fi
        if [[ $(echo $output | jq -r ".stats.sbxVersion") != "$ver" ]]; then
            echo "Invalid JSON"
            exit_code=1
        fi

        echo "Comparing decoded data to original"
        cmp dummy $output_name
        if [[ $? == 0 ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi
    done
done

exit $exit_code
