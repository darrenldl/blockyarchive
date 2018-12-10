#!/bin/bash
dd if=/dev/urandom of=dummy bs=$[1024 * 1024 * 1] count=1 &>/dev/null
