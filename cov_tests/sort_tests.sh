#!/bin/bash

source kcov_rsbx_fun.sh

exit_code=0

VERSIONS=(1 2 3 17 18 19)

file_size=$[1024 * 1024 * 10]

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 1; i++ )); do
        if   [[ $ver ==  1 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver ==  2 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver ==  3 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver == 17 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        elif [[ $ver == 18 ]]; then
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        else
            data_shards=$((1 + RANDOM % 128))
            parity_shards=$((1 + RANDOM % 128))
        fi

        burst=$((RANDOM % 15))

        container_name=sort_$data_shards\_$parity_shards\_$ver.sbx

        echo "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
        kcov_rsbx encode --sbx-version $ver -f dummy $container_name \
               --hash sha1 \
               --rs-data $data_shards --rs-parity $parity_shards &>/dev/null

        echo "Sorting container"
        kcov_rsbx sort --burst $burst $container_name sorted_$container_name \
            &>/dev/null

        output_name=dummy_$data_shards\_$parity_shards

        echo "Decoding"
        kcov_rsbx decode -f sorted_$container_name $output_name &>/dev/null

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
