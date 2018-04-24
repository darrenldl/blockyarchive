#!/bin/bash

exit_code=0

VERSIONS=(17 18 19)

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

        container_name=truncated_$data_shards\_$parity_shards\_$ver.sbx

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

        if   [[ $ver == 17 ]]; then
            block_size=512
        elif [[ $ver == 18 ]]; then
            block_size=128
        else
            block_size=4096
        fi

        actual_container_size=$(ls -l $container_name | awk '{print $5}')

        truncated_container_size=$(($actual_container_size
                                    - $parity_shards * $block_size))

        echo "Truncating container from $file_size to $truncated_file_size"
        truncate -s $truncated_container_size $container_name

        echo "Repairing"
        output=$(./rsbx repair --json --verbose $container_name 2>/dev/null)
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
