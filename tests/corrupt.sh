#!/bin/bash

function corrupt() {
    byte_orig=$(mktemp)
    byte_cur=$(mktemp)
    dd if=$2 of=$byte_orig bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
    while true; do
        dd if=/dev/urandom of=$2 bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
        dd if=$2 of=$byte_cur bs=1 count=1 seek=$1 conv=notrunc &>/dev/null
        cmp $byte_cur $byte_orig &>/dev/null
        if [[ $? != 0 ]]; then
            break
        fi
    done
}
