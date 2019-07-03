#!/bin/bash
dd if=/dev/urandom of=dummy bs=$[1024 * 2] count=2 &>/dev/null
file_size=$[(2 + RANDOM % 5) * 1024 + RANDOM % 1000]
echo $file_size > dummy_file_size
truncate -s $file_size dummy
