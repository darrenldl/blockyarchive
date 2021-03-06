#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

file_size=$(ls -l dummy | awk '{ print $5 }')

# generate test data
dd if=/dev/urandom of=dummy bs=$file_size count=1 &>/dev/null

for ver in ${VERSIONS[*]}; do
  for (( i=0; i < 3; i++ )); do
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

    burst=$[1 + RANDOM % 1000]

    # check that blkar defaults to guessing from start if --guess-burst-from is not specified
    offset=$[1 + RANDOM % 100]

    container_name=show_$data_shards\_$parity_shards\_$ver.sbx

    echo -n "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
    output=$(./../blkar encode --json --sbx-version $ver -f dummy $container_name \
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

    mv $container_name $container_name.tmp
    touch $container_name
    truncate -s $offset $container_name
    cat $container_name.tmp >> $container_name
    rm $container_name.tmp

    echo -n "Checking container burst error resistance level"
    output=$(./../blkar show --json --force-misalign --from $offset --guess-burst $container_name)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    burst_shown=$(echo $output | jq -r ".bestGuessForBurstErrorResistanceLevel")
    if [[ (($ver == "1" || $ver == "2" || $ver == "3") && ($burst_shown == "null"))
              || (($ver == "17" || $ver == "18" || $ver == "19") && ($burst_shown == $burst)) ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    # check that blkar moves to the specified location if --guess-burst-from is specified
    offset=$[500 + RANDOM % 1000]

    echo -n "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
    output=$(./../blkar encode --json --sbx-version $ver -f dummy $container_name \
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

    mv $container_name $container_name.tmp
    touch $container_name
    truncate -s $offset $container_name
    cat $container_name.tmp >> $container_name
    rm $container_name.tmp

    echo -n "Checking container burst error resistance level"
    output=$(./../blkar show --json --force-misalign --guess-burst-from $offset --from $offset --guess-burst $container_name)
    if [[ $(echo $output | jq -r ".error") != null ]]; then
      echo " ==> Invalid JSON"
      exit_code=1
    fi
    burst_shown=$(echo $output | jq -r ".bestGuessForBurstErrorResistanceLevel")
    if [[ (($ver == "1" || $ver == "2" || $ver == "3") && ($burst_shown == "null"))
              || (($ver == "17" || $ver == "18" || $ver == "19") && ($burst_shown == $burst)) ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi
  done
done

echo $exit_code > exit_code
