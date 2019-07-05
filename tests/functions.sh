#!/bin/bash

corrupt() {
    byte_orig=$(mktemp)
    byte_cur=$(mktemp)
    dd if=$2 of=$byte_orig bs=1 count=1 skip=$1 &>/dev/null
    while true; do
        dd if=/dev/urandom of=$2        bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
        dd if=$2           of=$byte_cur bs=1 count=1 skip=$1 &>/dev/null
        cmp $byte_cur $byte_orig &>/dev/null
        if [[ $? != 0 ]]; then
            break
        fi
    done
    rm $byte_orig
    rm $byte_cur
}

burst_corrupt() {
    block_size=$2
    start_pos=$(( $1 / $block_size * $block_size ))
    burst=$3
    file=$4
    for i in $(seq 0 $[burst - 1]); do
        pos=$(( $start_pos + $i * $block_size ))
        corrupt $pos $file
    done
}
