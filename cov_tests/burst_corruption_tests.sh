#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

VERSIONS=(17 18 19)

corrupt() {
    dd if=/dev/zero of=$4 bs=$2 count=$3 seek=$1 conv=notrunc &>/dev/null
}

file_size=$[1024 * 1024 * 1]

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 3; i++ )); do
        if   [[ $ver == 17 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
            burst=$((1 + RANDOM % 10))
        elif [[ $ver == 18 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
            burst=$((1 + RANDOM % 10))
        else
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
            burst=$((1 + RANDOM % 10))
        fi

        container_name=burst_$data_shards\_$parity_shards\_$burst\_$ver.sbx

        kcov_rsbx encode --sbx-version $ver -f dummy $container_name \
               --hash sha1 \
               --rs-data $data_shards --rs-parity $parity_shards \
               --burst $burst &>/dev/null

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

        echo "Repairing"
        kcov_rsbx repair -y $container_name &>/dev/null

        output_name=dummy_$data_shards\_$parity_shards

        echo "Decoding"
        kcov_rsbx decode -f $container_name $output_name &>/dev/null

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
