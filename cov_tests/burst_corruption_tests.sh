#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

VERSIONS=(17)

corrupt() {
    dd if=/dev/zero of=$4 bs=$2 count=$3 seek=$1 conv=notrunc &>/dev/null
}

file_size=$[1024 * 2]

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 1; i++ )); do
        if   [[ $ver == 17 ]]; then
            data_shards=$((1 + RANDOM % 5))
            parity_shards=$((1 + RANDOM % 5))
            burst=$((1 + RANDOM % 10))
        elif [[ $ver == 18 ]]; then
            data_shards=$((1 + RANDOM % 5))
            parity_shards=$((1 + RANDOM % 5))
            burst=$((1 + RANDOM % 10))
        else
            data_shards=$((1 + RANDOM % 5))
            parity_shards=$((1 + RANDOM % 5))
            burst=$((1 + RANDOM % 10))
        fi

        container_name=burst_$data_shards\_$parity_shards\_$burst\_$ver.sbx

        echo -n "Encoding"
        output=$(kcov_rsbx encode --json --sbx-version $ver -f dummy $container_name \
                           --hash sha1 \
                           --rs-data $data_shards --rs-parity $parity_shards \
                           --burst $burst)
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
        if [[ $(echo $output | jq -r ".stats.hash" | awk '{ print $1 }') == "SHA1" ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        if   [[ $ver == 17 ]]; then
            block_size=512
        elif [[ $ver == 18 ]]; then
            block_size=128
        else
            block_size=4096
        fi

        echo "Corrupting at $parity_shards random positions, burst error size is $burst"
        for (( p=0; p < $parity_shards; p++ )); do
            pos=$(( (RANDOM % $file_size) / $block_size ))
            # echo "#$p corruption, corrupting byte at position : $pos"
            corrupt $pos $block_size $burst $container_name
        done

        echo -n "Repairing"
        output=$(kcov_rsbx repair --json --verbose $container_name)
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

        output_name=dummy_$data_shards\_$parity_shards

        echo -n "Decoding"
        output=$(kcov_rsbx decode --json -f $container_name $output_name)
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

        echo -n "Comparing decoded data to original"
        cmp dummy $output_name
        if [[ $? == 0 ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi
    done
done

exit $exit_code
