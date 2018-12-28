#!/bin/bash

exit_code=0

file_size=$[1024 * 1024 * 10]

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in 1 2 3; do
    for (( i=0; i < 3; i++ )); do
        burst=$((RANDOM % 15))

        container_name=sort_$ver.sbx

        echo -n "Encoding in version $ver"
        output=$(./blkar encode --json --sbx-version $ver -f dummy $container_name \
                        --hash sha1)
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

        new_burst=$[$burst+2]

        echo -n "Sorting container"
        output=$(./blkar sort --json -f --burst $new_burst $container_name sorted_$container_name)
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

        meta_same_order=$(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata")
        meta_diff_order=$(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata")
        data_same_order=$(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData")
        data_diff_order=$(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData")

        if [[ $meta_same_order == 1 ]]; then
          echo -n " ==> Okay"
        else
          echo -n " ==> NOT okay"
          exit_code=1
        fi
        if [[ $meta_diff_order == 0 ]]; then
          echo -n " ==> Okay"
        else
          echo -n " ==> NOT okay"
          exit_code=1
        fi
        if [[ $data_same_order > 0 ]]; then
          echo -n " ==> Okay"
        else
          echo -n " ==> NOT okay"
          exit_code=1
        fi
        if [[ $data_diff_order == 0 ]]; then
          echo " ==> Okay"
        else
          echo " ==> NOT okay"
          exit_code=1
        fi
    done
done

for ver in 17 18 19; do
  for (( i=0; i < 3; i++ )); do
    burst=$((1 + RANDOM % 15))

    data_shards=$((1 + RANDOM % 128))
    parity_shards=$((1 + RANDOM % 128))

    container_name=sort_$data_shards\_$parity_shards\_$ver.sbx

    echo -n "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
    output=$(./blkar encode --json --sbx-version $ver -f dummy $container_name \
                     --hash sha1 \
                     --burst $burst \
                     --rs-data $data_shards --rs-parity $parity_shards)
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

    echo -n "Sorting container with same burst error resistance level"
    output=$(./blkar sort --json -f --burst $burst $container_name sorted_$container_name)
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

    meta_same_order=$(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata")
    meta_diff_order=$(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata")
    data_same_order=$(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData")
    data_diff_order=$(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData")

    if [[ $meta_same_order == $[1 + $parity_shards] ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $meta_diff_order == 0 ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $data_same_order > 0 ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $data_diff_order == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    new_burst=$[burst * (data_shards + parity_shards)]

    echo -n "Sorting container with different burst error resistance level"
    output=$(./blkar sort --json -f --burst $new_burst $container_name sorted_$container_name)
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

    meta_same_order=$(echo $output | jq -r ".stats.numberOfBlocksInSameOrderMetadata")
    meta_diff_order=$(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderMetadata")
    data_same_order=$(echo $output | jq -r ".stats.numberOfBlocksInSameOrderData")
    data_diff_order=$(echo $output | jq -r ".stats.numberOfBlocksInDiffOrderData")

    if [[ $meta_same_order == 1 ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $meta_diff_order == $parity_shards ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $data_same_order > 0 ]]; then
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi
    if [[ $data_diff_order > 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi
  done
done

exit $exit_code
