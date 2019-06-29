#!/bin/bash
dd if=/dev/urandom of=dummy bs=$[1024 * 1024] count=2 &>/dev/null
file_size=$[(1024 + RANDOM % 1000) + RANDOM % 1000]
echo "Using dummy file of size :" $file_size
truncate -s $file_size dummy
