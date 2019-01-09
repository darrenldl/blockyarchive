#!/bin/bash

exit_code=0

VERSIONS=(1 2 3 17 18 19)

corrupt() {
  dd if=/dev/zero of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
}

file_size=$(ls -l dummy | awk '{ print $5 }')

rm -rf dummy_blank
dd if=/dev/zero of=dummy_blank bs=1024 count=$[file_size / 1024] 2>/dev/null

# # Encode in all 6 versions
# for ver in ${VERSIONS[*]}; do
#   echo -n "Encoding in version $ver"
#   output=$(./../blkar encode --json --sbx-version $ver -f dummy dummy$ver.sbx \
#                    --rs-data 10 --rs-parity 2)
#   if [[ $(echo $output | jq -r ".error") != null ]]; then
#     echo " ==> Invalid JSON"
#     exit_code=1
#   fi
#   if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
#     echo " ==> Okay"
#   else
#     echo " ==> NOT okay"
#     exit_code=1
#   fi
# done

# # Create corrupted copies
# echo "Creating corrupted copies"
# for ver in ${VERSIONS[*]}; do
#   cp dummy$ver.sbx dummy$ver.1.sbx
#   cp dummy$ver.sbx dummy$ver.2.sbx
#   cp dummy$ver.sbx dummy$ver.3.sbx
#   cp dummy$ver.sbx dummy$ver.4.sbx
#   cp dummy$ver.sbx dummy$ver.5.sbx
#   mv dummy$ver.sbx dummy$ver.6.sbx

#   corrupt  5000 dummy$ver.1.sbx
#   corrupt 10000 dummy$ver.1.sbx
#   corrupt 15000 dummy$ver.1.sbx
#   corrupt 20000 dummy$ver.1.sbx

#   corrupt 10000 dummy$ver.2.sbx
#   corrupt 15000 dummy$ver.2.sbx
#   corrupt 20000 dummy$ver.2.sbx

#   corrupt 15000 dummy$ver.3.sbx
#   corrupt 20000 dummy$ver.3.sbx

#   corrupt 20000 dummy$ver.4.sbx

#   corrupt  5000 dummy$ver.5.sbx
#   corrupt 10000 dummy$ver.5.sbx
#   corrupt 15000 dummy$ver.5.sbx

#   corrupt  5000 dummy$ver.6.sbx
#   corrupt 10000 dummy$ver.6.sbx
#   corrupt 15000 dummy$ver.6.sbx
#   corrupt 20000 dummy$ver.6.sbx
# done

# # Decode all of them
# for ver in ${VERSIONS[*]}; do
#   echo "Decoding version $ver container"
#   rm -f dummy$ver
#   for i in 1 2 3 4 5 6; do
#     echo -n "    pass $i"
#     output=$(./../blkar decode --json --verbose --multi-pass-no-skip dummy$ver.$i.sbx dummy$ver)
#     if [[ $(echo $output | jq -r ".error") != null ]]; then
#       echo " ==> Invalid JSON"
#       exit_code=1
#     fi
#     if [[ $(echo $output | jq -r ".stats.sbxVersion") == "$ver" ]]; then
#       echo -n " ==> Okay"
#     else
#       echo -n " ==> NOT okay"
#       exit_code=1
#     fi
#     if [[ $i < 5 ]]; then
#       if [[ $(echo $output | jq -r ".stats.recordedHash") != $(echo $output | jq -r ".stats.hashOfOutputFile") ]]; then
#         echo " ==> Okay"
#       else
#         echo " ==> NOT okay"
#         exit_code=1
#       fi
#     else
#       if [[ $(echo $output | jq -r ".stats.recordedHash") == $(echo $output | jq -r ".stats.hashOfOutputFile") ]]; then
#         echo " ==> Okay"
#       else
#         echo " ==> NOT okay"
#         exit_code=1
#       fi
#     fi
#   done
# done

# # Compare to original file
# for ver in ${VERSIONS[*]}; do
#   echo -n "Comparing decoded version $ver container data to original"
#   cmp dummy dummy$ver
#   if [[ $? == 0 ]]; then
#     echo " ==> Okay"
#   else
#     echo " ==> NOT okay"
#     exit_code=1
#   fi
# done

for ver in ${VERSIONS[*]}; do
  for (( i=0; i < 1; i++ )); do
    data_shards=10
    parity_shards=2
    if   [[ $ver ==  1 ]]; then
      block_size=512
      meta_count=1
    elif [[ $ver ==  2 ]]; then
      block_size=128
      meta_count=1
    elif [[ $ver ==  3 ]]; then
      block_size=4096
      meta_count=1
    elif [[ $ver == 17 ]]; then
      block_size=512
      meta_count=$[parity_shards + 1]
    elif [[ $ver == 18 ]]; then
      block_size=128
      meta_count=$[parity_shards + 1]
    else
      block_size=4096
      meta_count=$[parity_shards + 1]
    fi

    chunk_size=$[block_size - 16]

    burst=$((RANDOM % 15))

    container_name=decode_$data_shards\_$parity_shards\_$ver.sbx

    echo -n "Encoding in version $ver, data = $data_shards, parity = $parity_shards"
    output=$(./../blkar encode --json --sbx-version $ver -f dummy $container_name.1 \
                        --uid DEADBEEF0001 \
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
      echo -n " ==> Okay"
    else
      echo -n " ==> NOT okay"
      exit_code=1
    fi

    output=$(./../blkar encode --json --sbx-version $ver -f dummy_blank $container_name.2 \
                        --uid DEADBEEF0001 \
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

    output_name=dummy_$data_shards\_$parity_shards

    echo -n "Decoding using 1st container"
    output=$(./../blkar decode --json -f $container_name.1 $output_name.1)
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

    echo "Decoding using 2nd container"
    output=$(./../blkar decode --json -f $container_name.2 $output_name.2)
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

    output=$(./../blkar decode --json --multi-pass-no-skip $container_name.2 $output_name.1)
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

    echo -n "Checking container block source"
    cmp $output_name.1 $output_name.2 >/dev/null
    if [[ $? == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi

    echo -n "Comparing decoded data to original"
    cmp dummy $output_name.2 >/dev/null
    if [[ $? == 0 ]]; then
      echo " ==> Okay"
    else
      echo " ==> NOT okay"
      exit_code=1
    fi
  done
done

echo $exit_code > exit_code
