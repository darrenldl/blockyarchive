#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

touch dummy

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 10; i++ )); do
        actual_file_size=$((RANDOM % 4096))
        truncate -s $actual_file_size dummy

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

        burst=$((1 + RANDOM % 10))

        echo "Testing for version $ver, data = $data_shards, parity = $parity_shards, burst = $burst"

        encode_info_file_size=$(./rsbx encode --info-only --sbx-version $ver dummy \
                                       --rs-data $data_shards --rs-parity $parity_shards \
                                       --burst $burst \
                                    | grep "File size" \
                                    | awk '{print $4}')

        encode_info_container_size=$(./rsbx encode --info-only --sbx-version $ver dummy \
                                            --rs-data $data_shards --rs-parity $parity_shards \
                                            --burst $burst \
                                         | grep "SBX container size" \
                                         | awk '{print $5}')

        calc_mode_file_size=$(./rsbx calc $actual_file_size --sbx-version $ver \
                                     --rs-data $data_shards --rs-parity $parity_shards \
                                     --burst $burst \
                                  | grep "File size" \
                                  | awk '{print $4}')

        calc_mode_container_size=$(./rsbx calc $actual_file_size --sbx-version $ver \
                                          --rs-data $data_shards --rs-parity $parity_shards \
                                          --burst $burst \
                                       | grep "SBX container size" \
                                       | awk '{print $5}')

        encode_stats_file_size=$(./rsbx encode --sbx-version $ver -f dummy \
                                        --hash sha1 \
                                        --rs-data $data_shards --rs-parity $parity_shards \
                                        --burst $burst \
                                     | grep "File size" \
                                     | awk '{print $4}')

        encode_stats_container_size=$(./rsbx encode --sbx-version $ver -f dummy \
                                             --hash sha1 \
                                             --rs-data $data_shards --rs-parity $parity_shards \
                                             --burst $burst \
                                          | grep "SBX container size" \
                                          | awk '{print $5}')

        actual_container_size=$(ls -l dummy.sbx | awk '{print $5}')

        echo "Checking if encode --info-only file size matches actual file size"
        if [[ $actual_file_size == $encode_info_file_size ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi

        echo "Checking if calc mode file size matches actual file size"
        if [[ $actual_file_size == $calc_mode_file_size ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi

        echo "Checking if encode mode stats file size matches actual file size"
        if [[ $actual_file_size == $encode_stats_file_size ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi

        echo "Checking if encode --info-only container size matches actual container size"
        if [[ $actual_container_size == $encode_info_container_size ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi

        echo "Checking if calc mode container size matches actual container size"
        if [[ $actual_container_size == $calc_mode_container_size ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi

        echo "Checking if encode mode stats container size matches actual container size"
        if [[ $actual_container_size == $encode_stats_container_size ]]; then
            echo "==> Okay"
        else
            echo "==> NOT okay"
            exit_code=1
        fi
    done
done

exit $exit_code
