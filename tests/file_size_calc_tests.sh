#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

touch dummy

for ver in ${VERSIONS[*]}; do
    for (( i=0; i < 3; i++ )); do
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

        burst=$((RANDOM % 15))

        echo "Testing for version $ver, data = $data_shards, parity = $parity_shards, burst = $burst"

        output=$(./rsbx encode --json --info-only --sbx-version $ver dummy \
                        --rs-data $data_shards --rs-parity $parity_shards \
                        --burst $burst)
        if [[ $(echo $output | jq -r ".error") != null ]]; then
            echo " ==> Invalid JSON"
            exit_code=1
        fi
        encode_info_file_size=$(echo $output | jq -r ".stats.fileSize")

        encode_info_container_size=$(echo $output | jq -r ".stats.sbxContainerSize")

        output=$(./rsbx calc --json $actual_file_size --sbx-version $ver \
                        --rs-data $data_shards --rs-parity $parity_shards \
                        --burst $burst)
        if [[ $(echo $output | jq -r ".error") != null ]]; then
            echo " ==> Invalid JSON"
            exit_code=1
        fi
        calc_mode_file_size=$(echo $output | jq -r ".stats.fileSize")

        calc_mode_container_size=$(echo $output | jq -r ".stats.sbxContainerSize")

        output=$(./rsbx encode --json --sbx-version $ver -f dummy \
                        --hash sha1 \
                        --rs-data $data_shards --rs-parity $parity_shards \
                        --burst $burst)
        if [[ $(echo $output | jq -r ".error") != null ]]; then
            echo " ==> Invalid JSON"
            exit_code=1
        fi
        encode_stats_file_size=$(echo $output | jq -r ".stats.fileSize")

        encode_stats_container_size=$(echo $output | jq -r ".stats.sbxContainerSize")

        actual_container_size=$(ls -l dummy.sbx | awk '{print $5}')

        echo -n "Checking if encode --info-only file size matches actual file size"
        if [[ $actual_file_size == $encode_info_file_size ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        echo -n "Checking if calc mode file size matches actual file size"
        if [[ $actual_file_size == $calc_mode_file_size ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        echo -n "Checking if encode mode stats file size matches actual file size"
        if [[ $actual_file_size == $encode_stats_file_size ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        echo -n "Checking if encode --info-only container size matches actual container size"
        if [[ $actual_container_size == $encode_info_container_size ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        echo -n "Checking if calc mode container size matches actual container size"
        if [[ $actual_container_size == $calc_mode_container_size ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi

        echo -n "Checking if encode mode stats container size matches actual container size"
        if [[ $actual_container_size == $encode_stats_container_size ]]; then
            echo " ==> Okay"
        else
            echo " ==> NOT okay"
            exit_code=1
        fi
    done
done

exit $exit_code
